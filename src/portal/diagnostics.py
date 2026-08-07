import os
import shutil
import subprocess
from dataclasses import dataclass, field
from typing import Callable

from portal.windows_provider import WindowSharingProvider


@dataclass
class PortalDiagnosticsReport:
    is_wayland: bool = False
    compositor: str = ""
    supports_window_sharing: bool = False
    window_sharing_reason: str | None = None
    pipewire_available: bool = False
    xdg_desktop_portal_active: bool = False
    xdg_desktop_portal_wlr_active: bool = False
    session_vars_exported: bool = False
    errors: list[str] = field(default_factory=list)

    def is_ready(self) -> bool:
        return (
            self.is_wayland
            and self.compositor in ("sway", "swayfx")
            and self.pipewire_available
            and self.xdg_desktop_portal_active
            and self.xdg_desktop_portal_wlr_active
            and self.session_vars_exported
        )


class PortalDiagnostics:
    """Checks whether the environment can share the screen through the portal."""

    def __init__(
        self,
        run: Callable[..., subprocess.CompletedProcess] = subprocess.run,
    ) -> None:
        self._run = run
        self._window_provider = WindowSharingProvider(run=run)

    def run(self) -> PortalDiagnosticsReport:
        report = PortalDiagnosticsReport()

        report.is_wayland = self._check_wayland()
        report.compositor = self._detect_compositor()
        window_availability = self._window_provider.get_availability()
        report.supports_window_sharing = window_availability.supported
        report.window_sharing_reason = window_availability.reason
        report.pipewire_available = self._check_pipewire()
        report.xdg_desktop_portal_active = self._check_service("xdg-desktop-portal")
        report.xdg_desktop_portal_wlr_active = self._check_service("xdg-desktop-portal-wlr")
        report.session_vars_exported = self._check_session_vars()

        if not report.is_wayland:
            report.errors.append("A sessão atual não utiliza Wayland.")
        if report.compositor not in ("sway", "swayfx"):
            report.errors.append("O compositor atual não é Sway ou SwayFX.")
        if not report.pipewire_available:
            report.errors.append("PipeWire não está disponível.")
        if not report.xdg_desktop_portal_active:
            report.errors.append("O serviço xdg-desktop-portal não está ativo.")
        if not report.xdg_desktop_portal_wlr_active:
            report.errors.append("O serviço xdg-desktop-portal-wlr não está ativo.")
        if not report.session_vars_exported:
            report.errors.append(
                "Variáveis de sessão não foram exportadas para o D-Bus "
                "(WAYLAND_DISPLAY ou XDG_CURRENT_DESKTOP podem estar faltando)."
            )
        if report.window_sharing_reason:
            report.errors.append(report.window_sharing_reason)

        return report

    def _check_wayland(self) -> bool:
        return os.environ.get("XDG_SESSION_TYPE", "").lower() == "wayland" or bool(
            os.environ.get("WAYLAND_DISPLAY")
        )

    def _detect_compositor(self) -> str:
        try:
            result = self._run(
                ["sway", "--version"],
                capture_output=True,
                text=True,
            )
            if result.returncode == 0:
                text = result.stdout.lower()
                if "swayfx" in text:
                    return "swayfx"
                if "sway" in text:
                    return "sway"
        except (OSError, subprocess.SubprocessError):
            pass

        desktop = os.environ.get("XDG_CURRENT_DESKTOP", "").lower()
        return "sway" if "sway" in desktop else ""

    def _check_pipewire(self) -> bool:
        return shutil.which("pipewire") is not None or shutil.which("pw-cli") is not None

    def _check_service(self, name: str) -> bool:
        try:
            result = self._run(
                ["systemctl", "--user", "is-active", name],
                capture_output=True,
                text=True,
            )
            return result.returncode == 0
        except (OSError, subprocess.SubprocessError):
            return False

    def _check_session_vars(self) -> bool:
        required = ("WAYLAND_DISPLAY", "XDG_CURRENT_DESKTOP")
        optional = ("SWAYSOCK", "XDG_SESSION_TYPE", "XDG_SESSION_DESKTOP")
        if not all(os.environ.get(v) for v in required):
            return False
        return any(os.environ.get(v) for v in optional)
