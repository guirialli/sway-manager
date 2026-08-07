import subprocess
from typing import Callable

from portal.models import PortalSource, PortalSourceType, WindowSharingAvailability
from portal.exceptions import WindowDiscoveryError


class WindowSharingProvider:
    """Discovers shareable windows.

    Sway 1.12 (2026-05-25) added official support for individual window capture
    through ext-foreign-toplevel-list-v1.  On older compositors the provider
    reports that window sharing is unavailable and returns an empty list.

    The first implementation delegates discovery to ``lswt`` when it is
    available.  The exact identifier returned to the portal is the one produced
    by that tool, not the window title or app_id.
    """

    MINIMUM_SWAY_VERSION_MAJOR = 1
    MINIMUM_SWAY_VERSION_MINOR = 12

    def __init__(
        self,
        run: Callable[..., subprocess.CompletedProcess] = subprocess.run,
    ) -> None:
        self._run = run

    def get_availability(self) -> WindowSharingAvailability:
        """Describe whether this session can enumerate capture-safe windows."""
        version = self._get_sway_version()
        if version is None:
            return WindowSharingAvailability(
                supported=False,
                reason=(
                    "Não foi possível detectar a versão do Sway. "
                    "A captura de janelas requer Sway 1.12 ou superior."
                ),
            )

        if not self._supports_version(version):
            return WindowSharingAvailability(
                supported=False,
                reason=(
                    f"Este compositor é baseado no Sway {version[0]}.{version[1]}. "
                    "A captura individual de janelas requer Sway 1.12 ou superior."
                ),
            )

        if not self._has_lswt():
            return WindowSharingAvailability(
                supported=False,
                reason=(
                    "O utilitário lswt não está instalado. Instale-o para listar "
                    "janelas compartilháveis."
                ),
            )

        return WindowSharingAvailability(supported=True)

    def is_supported(self) -> bool:
        """Return whether the compositor supports individual window capture."""
        version = self._get_sway_version()
        return version is not None and self._supports_version(version)

    def get_windows(
        self, availability: WindowSharingAvailability | None = None
    ) -> list[PortalSource]:
        availability = availability or self.get_availability()
        if not availability.supported:
            return []

        try:
            return self._list_windows_with_lswt()
        except (OSError, subprocess.SubprocessError) as exc:
            raise WindowDiscoveryError(f"falha ao consultar janelas: {exc}") from exc

    @classmethod
    def _supports_version(cls, version: tuple[int, int]) -> bool:
        major, minor = version
        return major > cls.MINIMUM_SWAY_VERSION_MAJOR or (
            major == cls.MINIMUM_SWAY_VERSION_MAJOR
            and minor >= cls.MINIMUM_SWAY_VERSION_MINOR
        )

    def _get_sway_version(self) -> tuple[int, int] | None:
        try:
            result = self._run(
                ["sway", "--version"],
                capture_output=True,
                text=True,
            )
        except (OSError, subprocess.SubprocessError):
            return None

        if result.returncode != 0:
            return None

        return self._parse_sway_version(result.stdout)

    @staticmethod
    def _parse_sway_version(text: str) -> tuple[int, int] | None:
        """Parse major/minor from strings like 'sway version 1.12' or SwayFX."""
        import re

        match = re.search(r"sway\s+(?:version\s+)?(\d+)\.(\d+)", text, re.IGNORECASE)
        if match:
            return int(match.group(1)), int(match.group(2))

        # SwayFX based on sway X.Y
        match = re.search(r"based on sway\s+(\d+)\.(\d+)", text, re.IGNORECASE)
        if match:
            return int(match.group(1)), int(match.group(2))

        return None

    def _has_lswt(self) -> bool:
        try:
            result = self._run(["which", "lswt"], capture_output=True, text=True)
            return result.returncode == 0 and bool(result.stdout.strip())
        except (OSError, subprocess.SubprocessError):
            return False

    def _list_windows_with_lswt(self) -> list[PortalSource]:
        import json
        import re

        result = self._run(
            ["lswt", "-j"],
            capture_output=True,
            text=True,
        )

        if result.returncode == 0 and result.stdout.strip():
            raw_stdout = result.stdout
            # Fix potential unquoted hex identifiers in lswt v2.0.0 JSON output:
            cleaned = re.sub(
                r'"(identifier|id)"\s*:\s*([a-fA-F0-9]+)\b',
                r'"\1": "\2"',
                raw_stdout,
            )
            try:
                data = json.loads(cleaned)
                toplevels = (
                    data.get("toplevels", [])
                    if isinstance(data, dict)
                    else data
                )
                if isinstance(toplevels, list):
                    sources: list[PortalSource] = []
                    for toplevel in toplevels:
                        if not isinstance(toplevel, dict):
                            continue
                        window_id = str(
                            toplevel.get("identifier")
                            or toplevel.get("id")
                            or ""
                        ).strip()
                        if not window_id:
                            continue

                        app_id = (
                            toplevel.get("app-id")
                            or toplevel.get("app_id")
                            or ""
                        ).strip()
                        title = (toplevel.get("title") or "").strip()
                        workspace = toplevel.get("workspace")

                        label = title or app_id or window_id
                        detail_lines = []
                        if app_id and app_id != title:
                            detail_lines.append(app_id)
                        if workspace:
                            detail_lines.append(f"Workspace {workspace}")

                        sources.append(
                            PortalSource(
                                id=window_id,
                                source_type=PortalSourceType.WINDOW,
                                label=label,
                                details="\n".join(detail_lines),
                                is_focused=bool(
                                    toplevel.get("activated")
                                    or toplevel.get("focused")
                                ),
                                extra={"toplevel": toplevel},
                            )
                        )
                    return sources
            except json.JSONDecodeError:
                pass

        # Fallback for lswt custom line output if JSON fails or returns non-zero
        alt_result = self._run(
            ["lswt", "-c", "iat"],
            capture_output=True,
            text=True,
        )
        if alt_result.returncode != 0:
            err = result.stderr.strip() or alt_result.stderr.strip()
            raise WindowDiscoveryError(f"lswt falhou: {err}")

        sources = []
        for line in alt_result.stdout.splitlines():
            line = line.strip()
            if not line:
                continue
            parts = line.split(",", 2)
            if not parts or not parts[0].strip():
                continue
            window_id = parts[0].strip()
            app_id = parts[1].strip() if len(parts) > 1 else ""
            title = parts[2].strip() if len(parts) > 2 else ""

            label = title or app_id or window_id
            detail_lines = []
            if app_id and app_id != title:
                detail_lines.append(app_id)

            sources.append(
                PortalSource(
                    id=window_id,
                    source_type=PortalSourceType.WINDOW,
                    label=label,
                    details="\n".join(detail_lines),
                )
            )

        return sources
