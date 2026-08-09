# SwayManager — Testing Specification

## 1. Overview & Test Framework

SwayManager uses Python's standard `unittest` framework combined with `unittest.mock`.

All test files are stored in `tests/` and named `test_<feature>.py`.

### Execution Command
To run the complete test suite reliably (including PySide6 GUI tests), execute:
```bash
PYTHONPATH=src QT_QPA_PLATFORM=offscreen venv/bin/python -m unittest discover -s tests -v
```

### Invariants:
1. **Headless Qt Testing**: `QT_QPA_PLATFORM=offscreen` MUST be set when running tests so PySide6 widgets instantiate headlessly without requiring an active X11/Wayland display server.
2. **Virtual Environment Python**: Use `venv/bin/python` where PySide6 is installed.
3. **Isolation**: Tests MUST NOT require a live Sway compositor session, hardware backlight, or root privileges. All external subprocess calls (`swaymsg`, `grim`, `slurp`, `lswt`, `wofi`, `cliphist`) are mocked.

---

## 2. Test Suite Structure & Catalog

```text
tests/
├── test_clipboard.py                # Cliphist parsing, favorites JSON, use case execution
├── test_config_repository.py         # JsonConfigRepository reading, writing, temp file isolation
├── test_daemon.py                   # SwayManagerDaemon startup, socket connection, IPC dispatch
├── test_display_repository.py        # SwayDisplayRepository output parsing, display switching
├── test_freeze_selection_overlay.py # Screenshot selection overlay rectangle math & scaling
├── test_lightdm_domain.py           # LightDMSettings serialization & default values
├── test_lightdm_repository.py       # LightDM GTK greeter INI parser & user/session filtering
├── test_lock_repository.py          # LockScreenUseCase & SwayLockRepository theme resolution
├── test_logging.py                  # AsyncLogger background queue & daily file writing
├── test_menu.py                     # DesktopParser .desktop parsing, icon resolver, Wofi launcher
├── test_notification_domain.py      # DesktopNotificationRepository & notification entity
├── test_portal.py                   # XDG Portal controller, outputs, windows, dialog, result writer
├── test_screenshot_repository.py   # GrimSlurpScreenshotRepository geometry parsing & fallback
├── test_sysfs_battery_repository.py # Sysfs battery conservation state toggling & waybar json
├── test_use_cases.py                # Application use cases execution
└── test_wallpaper_repository.py     # SwayWallpaperRepository folder fallback & config persistence
```

---

## 3. Key Mocking Strategies & Patterns

### 3.1 Mocking Subprocess Execution (`swaymsg`, `grim`, `slurp`)
Tests use `@patch("subprocess.run")` or `@patch("subprocess.Popen")` to return mock stdout payloads without executing real binaries:

```python
@patch("subprocess.run")
def test_parses_active_outputs(self, mock_run):
    mock_run.return_value.stdout = json.dumps([
        {"name": "eDP-1", "active": True, "focused": True, "rect": {"x": 0, "y": 0, "width": 1920, "height": 1080}},
        {"name": "HDMI-A-1", "active": False, "focused": False, "rect": {"x": 1920, "y": 0, "width": 1920, "height": 1080}}
    ])
    provider = SwayOutputsProvider()
    outputs = provider.get_active_outputs()
    self.assertEqual(len(outputs), 1)
    self.assertEqual(outputs[0].id, "eDP-1")
```

### 3.2 Mocking Sysfs & Filesystem Calls
Tests use `unittest.mock.mock_open` and `patch("os.path.exists")` to simulate sysfs nodes (e.g. `/sys/bus/platform/drivers/ideapad_acpi/VPC2004:00/conservation_mode`).

### 3.3 Test Environment Isolation
`JsonConfigRepository` automatically isolates config mutations to `/tmp/sway_manager_tests/test_config.json` during test runs, preventing modification of developer settings in `~/.config/sway-manager/config.json`.

---

## 4. Test Coverage Catalog & Gaps

### Covered Components (High Coverage):
- `portal` subsystem (controller, dialog, providers, writer, installer, diagnostics)
- `JsonConfigRepository` & `AsyncLogger`
- `SwayDisplayRepository` & `GrimSlurpScreenshotRepository`
- `FreezeSelectionOverlay` coordinate math & scale calculations
- `SysfsBatteryRepository` & `CliphistRepository`
- `DesktopParser`, `IconResolver`, and `WofiRepository`

### Coverage Gaps (Areas Recommended for Additional Tests):
1. **`MixerAudioRepository`**: Lacks direct unit tests for `pactl`/`wpctl`/`pamixer` volume parsing.
2. **`PowerProfilesRepository`**: Lacks tests for D-Bus / `powerprofilesctl` responses.
3. **`CLIHandlers` Integration**: `presentation/cli/handlers.py` is mostly exercised indirectly; explicit router argument parsing tests could be added.
