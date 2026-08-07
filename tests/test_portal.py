import io
import json
import os
import unittest
from pathlib import Path
from unittest.mock import MagicMock, patch

from portal.models import (
    PortalResult,
    PortalSource,
    PortalSourceType,
    WindowSharingAvailability,
)
from portal.result_writer import PortalResultWriter
from portal.outputs_provider import SwayOutputsProvider
from portal.windows_provider import WindowSharingProvider
from portal.diagnostics import PortalDiagnostics, PortalDiagnosticsReport
from portal.exceptions import SwayNotAvailableError
from portal.config_installer import PortalConfigInstaller


class MockCompletedProcess:
    def __init__(self, returncode=0, stdout="", stderr=""):
        self.returncode = returncode
        self.stdout = stdout
        self.stderr = stderr


def make_run_mock(returncode=0, stdout="", stderr=""):
    def _run(*args, **kwargs):
        return MockCompletedProcess(returncode=returncode, stdout=stdout, stderr=stderr)

    return _run


class TestPortalResultWriter(unittest.TestCase):
    def test_write_monitor(self):
        out = io.StringIO()
        writer = PortalResultWriter(out=out)
        writer.write_monitor("HDMI-A-1")
        self.assertEqual(out.getvalue(), "HDMI-A-1\n")

    def test_write_window(self):
        out = io.StringIO()
        writer = PortalResultWriter(out=out)
        writer.write_window("window-123")
        self.assertEqual(out.getvalue(), "window-123\n")

    def test_write_result(self):
        out = io.StringIO()
        writer = PortalResultWriter(out=out)
        writer.write_result(PortalResult(PortalSourceType.MONITOR, "eDP-1"))
        self.assertEqual(out.getvalue(), "eDP-1\n")

    def test_cancel_writes_nothing(self):
        out = io.StringIO()
        writer = PortalResultWriter(out=out)
        writer.cancel()
        self.assertEqual(out.getvalue(), "")

    def test_validate_identifier_rejects_multiline(self):
        self.assertTrue(PortalResultWriter.validate_identifier("HDMI-A-1"))
        self.assertFalse(PortalResultWriter.validate_identifier(""))
        self.assertFalse(PortalResultWriter.validate_identifier("line\nbreak"))


class TestSwayOutputsProvider(unittest.TestCase):
    def test_parses_active_outputs(self):
        data = [
            {
                "name": "eDP-1",
                "active": True,
                "focused": True,
                "primary": True,
                "make": "AU Optronics",
                "model": "B140HAN",
                "current_mode": {"width": 1920, "height": 1080, "refresh": 60000},
                "scale": 1.0,
                "transform": "normal",
                "rect": {"x": 0, "y": 0},
            },
            {
                "name": "HDMI-A-1",
                "active": False,
                "make": "Dell",
                "model": "U2720Q",
                "current_mode": {"width": 3840, "height": 2160, "refresh": 60000},
                "scale": 1.5,
                "rect": {"x": 1920, "y": 0},
            },
        ]
        run = make_run_mock(stdout=json.dumps(data))
        provider = SwayOutputsProvider(run=run)
        outputs = provider.get_outputs()

        self.assertEqual(len(outputs), 1)
        self.assertEqual(outputs[0].id, "eDP-1")
        self.assertEqual(outputs[0].source_type, PortalSourceType.MONITOR)
        self.assertIn("1920 × 1080", outputs[0].details)
        self.assertIn("Escala: 1.0", outputs[0].details)
        self.assertTrue(outputs[0].is_primary)

    def test_raises_when_swaymsg_fails(self):
        run = make_run_mock(returncode=1, stderr="não está rodando")
        provider = SwayOutputsProvider(run=run)
        with self.assertRaises(SwayNotAvailableError):
            provider.get_outputs()

    def test_raises_when_no_active_outputs(self):
        data = [{"name": "eDP-1", "active": False}]
        run = make_run_mock(stdout=json.dumps(data))
        provider = SwayOutputsProvider(run=run)
        with self.assertRaises(SwayNotAvailableError):
            provider.get_outputs()


class TestWindowSharingProvider(unittest.TestCase):
    def test_unsupported_sway_returns_empty(self):
        run = make_run_mock(stdout="sway version 1.11")
        provider = WindowSharingProvider(run=run)
        self.assertFalse(provider.is_supported())
        self.assertEqual(provider.get_windows(), [])

    def test_supported_sway_without_lswt_returns_empty(self):
        run = make_run_mock(stdout="sway version 1.12")
        provider = WindowSharingProvider(run=run)
        with patch.object(provider, "_has_lswt", return_value=False):
            self.assertTrue(provider.is_supported())
            self.assertEqual(provider.get_windows(), [])

    def test_availability_explains_unsupported_sway_version(self):
        provider = WindowSharingProvider(run=make_run_mock(stdout="sway version 1.11"))

        availability = provider.get_availability()

        self.assertFalse(availability.supported)
        self.assertIn("Sway 1.11", availability.reason)

    def test_availability_requires_lswt(self):
        provider = WindowSharingProvider(run=make_run_mock(stdout="sway version 1.12"))

        with patch.object(provider, "_has_lswt", return_value=False):
            availability = provider.get_availability()

        self.assertFalse(availability.supported)
        self.assertIn("lswt", availability.reason)

    def test_parses_lswt_output(self):
        toplevels = [
            {
                "id": "toplevel-1",
                "app_id": "org.mozilla.firefox",
                "title": "Firefox",
                "workspace": "2",
                "focused": True,
            }
        ]

        def side_effect(cmd, **kwargs):
            if cmd[0] == "sway":
                return MockCompletedProcess(returncode=0, stdout="sway version 1.12")
            if cmd[0] == "which":
                return MockCompletedProcess(returncode=0, stdout="/usr/bin/lswt")
            if cmd[0] == "lswt":
                return MockCompletedProcess(returncode=0, stdout=json.dumps(toplevels))
            return MockCompletedProcess(returncode=1)

        provider = WindowSharingProvider(run=side_effect)
        windows = provider.get_windows()
        self.assertEqual(len(windows), 1)
        self.assertEqual(windows[0].id, "toplevel-1")
        self.assertEqual(windows[0].source_type, PortalSourceType.WINDOW)
        self.assertTrue(windows[0].is_focused)

    def test_parse_sway_version_for_swayfx(self):
        text = "swayfx version 0.5.3-660c1197 (based on sway 1.11.0)"
        self.assertEqual(
            WindowSharingProvider._parse_sway_version(text),
            (1, 11),
        )


class TestPortalDiagnostics(unittest.TestCase):
    @patch.dict(
        "os.environ",
        {
            "XDG_SESSION_TYPE": "wayland",
            "WAYLAND_DISPLAY": "wayland-1",
            "XDG_CURRENT_DESKTOP": "sway",
            "SWAYSOCK": "/run/user/1000/sway-ipc.sock",
        },
        clear=True,
    )
    def test_ready_when_all_services_active(self):
        def side_effect(cmd, **kwargs):
            if cmd[0] == "sway":
                return MockCompletedProcess(returncode=0, stdout="sway version 1.12")
            if cmd[0] == "systemctl":
                return MockCompletedProcess(returncode=0)
            return MockCompletedProcess(returncode=1)

        diag = PortalDiagnostics(run=side_effect)
        with patch("shutil.which", side_effect=lambda name: name in ("pipewire", "pw-cli")):
            report = diag.run()

        self.assertTrue(report.is_wayland)
        self.assertEqual(report.compositor, "sway")
        self.assertTrue(report.pipewire_available)
        self.assertTrue(report.xdg_desktop_portal_active)
        self.assertTrue(report.xdg_desktop_portal_wlr_active)
        self.assertTrue(report.session_vars_exported)
        self.assertTrue(report.is_ready())


class TestPortalController(unittest.TestCase):
    def test_returns_selected_result(self):
        from PySide6.QtWidgets import QDialog, QApplication
        from portal.controller import PortalController

        os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
        app = QApplication.instance()
        if app is None:
            app = QApplication([])

        class FakeSignal:
            def __init__(self):
                self._handler = None

            def connect(self, handler):
                self._handler = handler

            def emit(self, value):
                if self._handler is not None:
                    self._handler(value)

        class AutoDialog(QDialog):
            source_selected = FakeSignal()

            def __init__(self, monitors, windows):
                super().__init__()
                self._monitors = monitors
                from PySide6.QtCore import QTimer

                QTimer.singleShot(10, self._accept)

            def _accept(self):
                self.source_selected.emit(
                    PortalResult(PortalSourceType.MONITOR, self._monitors[0].id)
                )
                self.accept()

        controller = PortalController(
            outputs_provider=type(
                "Outputs",
                (),
                {
                    "get_outputs": lambda self: [
                        PortalSource(
                            id="DP-1",
                            source_type=PortalSourceType.MONITOR,
                            label="DP-1",
                            details="",
                        )
                    ]
                },
            )(),
            windows_provider=type(
                "Windows",
                (),
                {
                    "get_availability": lambda self: WindowSharingAvailability(
                        supported=True
                    ),
                    "get_windows": lambda self, availability: [],
                },
            )(),
            dialog_factory=lambda m, w: AutoDialog(m, w),
        )
        result = controller.run()
        self.assertEqual(result, PortalResult(PortalSourceType.MONITOR, "DP-1"))

    def test_parses_dmenu_stdin_sources(self):
        from portal.controller import PortalController

        stdin_content = (
            "Monitor: eDP-1\n"
            "Monitor: HDMI-A-1 Goldstar 1920x1080\n"
            "Window: Firefox (abc123def456)\n"
            "Window: Discord - Call (7890abcdef12)\n"
        )
        monitors, windows = PortalController._parse_dmenu_sources(stdin_content)

        self.assertEqual(len(monitors), 2)
        self.assertEqual(monitors[0].id, "eDP-1")
        self.assertEqual(monitors[0].raw_label, "Monitor: eDP-1")
        self.assertEqual(monitors[1].id, "HDMI-A-1")
        self.assertEqual(monitors[1].details, "Goldstar 1920x1080")

        self.assertEqual(len(windows), 2)
        self.assertEqual(windows[0].id, "abc123def456")
        self.assertEqual(windows[0].label, "Firefox")
        self.assertEqual(windows[1].id, "7890abcdef12")
        self.assertEqual(windows[1].label, "Discord - Call")

    def test_dmenu_result_preserves_raw_label(self):
        from portal.controller import PortalController

        stdin_content = "Monitor: eDP-1\nWindow: Firefox (abc123)\n"
        monitors, windows = PortalController._parse_dmenu_sources(stdin_content)

        window_result = PortalResult(
            source_type=windows[0].source_type,
            id=windows[0].id,
            raw_label=windows[0].raw_label,
        )
        self.assertEqual(str(window_result), "Window: Firefox (abc123)")

        monitor_result = PortalResult(
            source_type=monitors[0].source_type,
            id=monitors[0].id,
            raw_label=monitors[0].raw_label,
        )
        self.assertEqual(str(monitor_result), "Monitor: eDP-1")
class TestSelectionCache(unittest.TestCase):
    def setUp(self):
        from portal.selection_cache import clear_cache

        clear_cache()

    def tearDown(self):
        from portal.selection_cache import clear_cache

        clear_cache()

    def _monitor(self, name="eDP-1"):
        return PortalSource(
            id=name,
            source_type=PortalSourceType.MONITOR,
            label=name,
            details="",
            raw_label=f"Monitor: {name}",
        )

    def _window(self, identifier="abc123", title="Firefox"):
        return PortalSource(
            id=identifier,
            source_type=PortalSourceType.WINDOW,
            label=title,
            details="",
            raw_label=f"Window: {title} ({identifier})",
        )

    def test_replay_returns_none_with_no_cache(self):
        from portal.selection_cache import try_replay

        self.assertIsNone(try_replay([self._monitor()], [self._window()]))

    def test_store_and_replay_same_sources(self):
        from portal.selection_cache import store_selection, try_replay

        monitors = [self._monitor("HDMI-A-1")]
        windows = [self._window("abc123", "Firefox")]

        result = PortalResult(
            source_type=PortalSourceType.WINDOW,
            id="abc123",
            raw_label="Window: Firefox (abc123)",
        )
        store_selection(result, monitors, windows)

        replayed = try_replay(monitors, windows)
        self.assertIsNotNone(replayed)
        self.assertEqual(str(replayed), "Window: Firefox (abc123)")
        self.assertEqual(replayed.source_type, PortalSourceType.WINDOW)
        self.assertEqual(replayed.id, "abc123")

    def test_different_sources_not_replayed(self):
        from portal.selection_cache import store_selection, try_replay

        monitors_a = [self._monitor("eDP-1")]
        windows_a = [self._window("abc", "Firefox")]
        result = PortalResult(PortalSourceType.WINDOW, "abc", raw_label="Window: Firefox (abc)")
        store_selection(result, monitors_a, windows_a)

        # different monitors → no replay
        monitors_b = [self._monitor("HDMI-A-1")]
        self.assertIsNone(try_replay(monitors_b, windows_a))

        # different windows → no replay
        windows_b = [self._window("def", "Discord")]
        self.assertIsNone(try_replay(monitors_a, windows_b))

    def test_clear_cache_prevents_replay(self):
        from portal.selection_cache import clear_cache, store_selection, try_replay

        monitors = [self._monitor()]
        result = PortalResult(PortalSourceType.MONITOR, "eDP-1", raw_label="Monitor: eDP-1")
        store_selection(result, monitors, [])

        clear_cache()
        self.assertIsNone(try_replay(monitors, []))

class TestPortalDialog(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        os.environ.setdefault("QT_QPA_PLATFORM", "offscreen")
        from PySide6.QtWidgets import QApplication

        cls.app = QApplication.instance() or QApplication([])

    def setUp(self):
        from presentation.gui.portal.dialog import PortalDialog

        self.PortalDialog = PortalDialog
        self.monitor = PortalSource(
            id="eDP-1",
            source_type=PortalSourceType.MONITOR,
            label="Monitor interno",
            details="eDP-1\n1920 × 1080",
        )
        self.window = PortalSource(
            id="window-1",
            source_type=PortalSourceType.WINDOW,
            label="Firefox",
            details="org.mozilla.firefox",
        )

    def test_requires_explicit_selection_before_sharing(self):
        dialog = self.PortalDialog([self.monitor], [self.window])

        self.assertIsNone(dialog._selected_card)
        self.assertFalse(dialog._share_btn.isEnabled())

    def test_keyboard_activation_selects_the_focused_source(self):
        dialog = self.PortalDialog([self.monitor], [self.window])
        selected: list[PortalResult] = []
        dialog.source_selected.connect(selected.append)

        dialog._cards[0].activated.emit()

        self.assertEqual(
            selected, [PortalResult(PortalSourceType.MONITOR, "eDP-1")]
        )

    def test_window_card_emits_the_window_identifier(self):
        dialog = self.PortalDialog([self.monitor], [self.window])
        selected: list[PortalResult] = []
        dialog.source_selected.connect(selected.append)

        dialog._tab_bar.setCurrentIndex(1)
        dialog._cards[1].activated.emit()

        self.assertEqual(
            selected, [PortalResult(PortalSourceType.WINDOW, "window-1")]
        )

    def test_uses_the_native_qt_theme_and_clears_selection_on_tab_change(self):
        dialog = self.PortalDialog([self.monitor], [self.window])

        dialog._cards[0].clicked.emit()
        self.assertTrue(dialog._share_btn.isEnabled())
        dialog._tab_bar.setCurrentIndex(1)

        self.assertIn("QDialog", dialog.styleSheet())
        self.assertIsNone(dialog._selected_card)
        self.assertFalse(dialog._share_btn.isEnabled())

    def test_window_tab_explains_missing_prerequisite(self):
        from PySide6.QtWidgets import QLabel

        reason = "Este compositor é baseado no Sway 1.11."
        dialog = self.PortalDialog(
            [self.monitor], [], window_sharing_reason=reason
        )

        dialog._tab_bar.setCurrentIndex(1)
        self.app.processEvents()

        self.assertTrue(dialog._tab_bar.isTabEnabled(1))
        self.assertFalse(dialog._share_btn.isEnabled())
        self.assertIn(
            reason, [label.text() for label in dialog.findChildren(QLabel)]
        )


class TestPortalConfigInstaller(unittest.TestCase):
    def setUp(self):
        self.tmp = Path("/tmp/test_portal_config_installer")
        self.tmp.mkdir(parents=True, exist_ok=True)

    def tearDown(self):
        import shutil

        shutil.rmtree(self.tmp, ignore_errors=True)

    def _make_installer(self, executable: Path):
        executable.touch()
        installer = PortalConfigInstaller(executable_path=str(executable))
        installer.home = self.tmp
        installer.bin_dir = self.tmp / ".config" / "sway" / "bin"
        return installer

    @patch.object(PortalConfigInstaller, "_restart_portal_services")
    def test_preserves_existing_unrelated_options(self, _restart):
        executable = self.tmp / "SwayManager"
        installer = self._make_installer(executable)

        portal_conf = self.tmp / ".config" / "xdg-desktop-portal" / "portals.conf"
        portal_conf.parent.mkdir(parents=True, exist_ok=True)
        portal_conf.write_text("[preferred]\ndefault=kde\norg.freedesktop.impl.portal.FileChooser=gtk\n")

        log = installer.install()
        self.assertTrue(any("Backup criado" in line for line in log))

        text = portal_conf.read_text()
        self.assertIn("default = gtk", text)
        self.assertIn("org.freedesktop.impl.portal.FileChooser = gtk", text)
        self.assertIn("org.freedesktop.impl.portal.ScreenCast = wlr", text)

    @patch.object(PortalConfigInstaller, "_restart_portal_services")
    def test_creates_wlr_config(self, _restart):
        executable = self.tmp / "SwayManager"
        installer = self._make_installer(executable)

        log = installer.install()
        self.assertTrue(any("xdg-desktop-portal-wlr" in line for line in log))

        wlr_conf = self.tmp / ".config" / "xdg-desktop-portal-wlr" / "config"
        self.assertTrue(wlr_conf.exists())
        text = wlr_conf.read_text()
        self.assertIn("chooser_type = dmenu", text)
        self.assertIn(f"chooser_cmd = {executable} portal", text)


if __name__ == "__main__":
    unittest.main()
