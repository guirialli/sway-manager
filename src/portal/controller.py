import re
import select
import sys
from typing import Callable

from PySide6.QtWidgets import QApplication, QDialog

from portal.models import PortalResult, PortalSource, PortalSourceType, WindowSharingAvailability
from portal.outputs_provider import SwayOutputsProvider
from portal.result_writer import PortalResultWriter
from portal.windows_provider import WindowSharingProvider
from portal.exceptions import PortalException, SwayNotAvailableError


class PortalController:
    """Coordinates discovery, UI, and result delivery for screen sharing."""

    def __init__(
        self,
        outputs_provider: SwayOutputsProvider | None = None,
        windows_provider: WindowSharingProvider | None = None,
        result_writer: PortalResultWriter | None = None,
        dialog_factory: Callable[[list[PortalSource], list[PortalSource]], QDialog] | None = None,
    ) -> None:
        self._outputs_provider = outputs_provider or SwayOutputsProvider()
        self._windows_provider = windows_provider or WindowSharingProvider()
        self._result_writer = result_writer or PortalResultWriter()
        self._dialog_factory = dialog_factory

    def run(self) -> PortalResult | None:
        """Load sources, show the selector, and return the chosen result."""
        stdin_content = self._maybe_read_dmenu_stdin()
        if stdin_content is not None:
            monitors, windows = self._parse_dmenu_sources(stdin_content)
            window_sharing_reason = None
        else:
            monitors = self._load_monitors()
            windows, window_sharing_reason = self._load_windows()

        app = QApplication.instance()
        owns_app = app is None
        if owns_app:
            app = QApplication(sys.argv)
        app.setDesktopFileName("sway.apps.portal-chooser")

        dialog = self._create_dialog(monitors, windows, window_sharing_reason)
        result: PortalResult | None = None

        def on_accepted(res: PortalResult) -> None:
            nonlocal result
            result = res

        dialog.finished.connect(lambda _: dialog.deleteLater())
        if hasattr(dialog, "source_selected"):
            dialog.source_selected.connect(on_accepted)

        dialog.exec()

        if owns_app:
            app.quit()
            del app

        return result

    def write_result(self, result: PortalResult | None) -> None:
        """Emit the result using the configured writer."""
        if result is None:
            self._result_writer.cancel()
        else:
            self._result_writer.write_result(result)

    def _load_monitors(self) -> list[PortalSource]:
        try:
            return self._outputs_provider.get_outputs()
        except SwayNotAvailableError:
            raise
        except Exception as exc:
            raise SwayNotAvailableError(f"falha ao consultar telas: {exc}") from exc

    def _load_windows(self) -> tuple[list[PortalSource], str | None]:
        try:
            availability: WindowSharingAvailability = (
                self._windows_provider.get_availability()
            )
            if not availability.supported:
                return [], availability.reason

            windows = self._windows_provider.get_windows(availability)
            if not windows:
                return [], "Nenhuma janela compartilhável foi encontrada nesta sessão."
            return windows, None
        except PortalException:
            # Keep monitor sharing functional when window discovery fails.
            return [], "Não foi possível consultar as janelas compartilháveis."

    def _create_dialog(
        self,
        monitors: list[PortalSource],
        windows: list[PortalSource],
        window_sharing_reason: str | None,
    ) -> QDialog:
        if self._dialog_factory is not None:
            return self._dialog_factory(monitors, windows)

        from presentation.gui.portal.dialog import PortalDialog

        return PortalDialog(
            monitors=monitors,
            windows=windows,
            window_sharing_reason=window_sharing_reason,
        )


    @staticmethod
    def _maybe_read_dmenu_stdin() -> str | None:
        """Return piped stdin content if any — used to detect dmenu mode.

        When xdg-desktop-portal-wlr v0.8.x invokes the chooser with
        chooser_type=dmenu, it writes a newline-separated list of source
        labels on stdin and closes the pipe.  Reading stdin back returns
        that data.  When stdin is a TTY (interactive, test, daemon
        context), returns None so the controller falls back to
        swaymsg/lswt discovery.
        """
        if sys.stdin.isatty():
            return None
        try:
            r, _, _ = select.select([sys.stdin], [], [], 0.05)
        except (OSError, ValueError):
            return None
        if not r:
            return None
        try:
            content = sys.stdin.read()
        except (OSError, ValueError):
            return None
        return content if content.strip() else None

    @staticmethod
    def _parse_dmenu_sources(
        content: str,
    ) -> tuple[list[PortalSource], list[PortalSource]]:
        """Parse dmenu label list emitted by xdg-desktop-portal-wlr v0.8.x.

        Each line is either:
          Monitor: <name> <description>
          Window: <title> (<identifier>)

        The exact line text (raw_label) is preserved on the matching
        PortalSource so the PortalResult echoing it back on stdout matches
        xdpw's exact-string comparison.
        """
        monitors: list[PortalSource] = []
        windows: list[PortalSource] = []
        for line in content.splitlines():
            stripped = line.rstrip()
            if not stripped.strip():
                continue
            if stripped.startswith("Monitor: "):
                rest = stripped[len("Monitor: "):]
                parts = rest.split(None, 1)
                name = parts[0] if parts else ""
                description = parts[1] if len(parts) > 1 else ""
                if name:
                    monitors.append(
                        PortalSource(
                            id=name,
                            source_type=PortalSourceType.MONITOR,
                            label=name,
                            details=description,
                            raw_label=stripped,
                        )
                    )
            elif stripped.startswith("Window: "):
                rest = stripped[len("Window: "):]
                m = re.search(r"\s*\(([a-fA-F0-9]+)\)\s*$", rest)
                if m:
                    identifier = m.group(1)
                    title = rest[: m.start()].strip()
                else:
                    identifier = rest.strip()
                    title = identifier
                if identifier:
                    windows.append(
                        PortalSource(
                            id=identifier,
                            source_type=PortalSourceType.WINDOW,
                            label=title or identifier,
                            details="",
                            raw_label=stripped,
                        )
                    )
        return monitors, windows
