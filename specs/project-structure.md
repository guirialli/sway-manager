# SwayManager — Project Structure Specification

## 1. Directory Hierarchy Overview

```text
sway-manager/
├── src/                          # Main Python application source code
│   ├── main.py                   # Main CLI entrypoint (SwayManager)
│   ├── gui_main.py               # Main GUI entrypoint (SwayManagerGUI)
│   ├── domain/                   # Domain entities, value objects, and repository interfaces
│   ├── application/              # Application use cases
│   ├── infrastructure/           # Systems integration (Sway, Sysfs, Subprocesses, Daemon, Config)
│   ├── presentation/             # CLI router & PySide6 GUI views, windows, OSDs
│   ├── portal/                   # Standalone XDG Desktop Portal screencast chooser module
│   └── utils/                    # Array, string, and exception utilities
├── tests/                        # Automated test suite (unittest framework)
├── specs/                        # Architectural memory and specifications (Markdown)
├── .agents/skills/               # Reusable AI agent operational procedures
├── udev/                         # System udev rules for non-root hardware access
│   └── 99-battery-conservation.rules
├── requirements.txt              # Runtime Python dependencies (PySide6, setuptools)
├── build.sh                      # Nuitka compilation script producing out/ main binaries
├── install.sh                    # Installation script (~/.config/sway/bin/sway-manager)
├── AGENTS.md                     # Core project repository guidelines
└── README.md                     # Project overview and user documentation
```

---

## 2. Package Breakdown & Responsibilities

### 2.1 `src/domain/` (Core Enterprise Logic)
Holds domain models, value objects, and abstract repository definitions. Divided into functional domain folders:

| Domain | Files | Responsibilities |
|---|---|---|
| `domain/audio/` | `value_objects.py`, `repositories.py` | Volume levels, mute state, `IAudioRepository` interface |
| `domain/clipboard/` | `entities.py`, `repositories.py` | `ClipboardItem` entity, action items, `IClipboardRepository` |
| `domain/display/` | `entities.py`, `repositories.py` | `MonitoresSway`, `DisplaySwitchType` enum, `IDisplayRepository`, `IBrightnessRepository` |
| `domain/media/` | `value_objects.py`, `repositories.py` | `ScreenshotMode` enum (FULL, AREA, WINDOW), `IScreenshotRepository` |
| `domain/menu/` | `entities.py`, `repositories.py` | `MenuItem` entity, category filtering, `IMenuRepository` |
| `domain/notification/` | `entities.py`, `repositories.py` | `Notification` entity, priority levels, `INotificationRepository` |
| `domain/power/` | `entities.py`, `repositories.py` | `BatteryState`, `IdleState`, `PowerProfileState`, `IPowerRepository`, `IBatteryRepository`, `IIdleRepository`, `ILockRepository` |
| `domain/theme/` | `entities.py`, `repositories.py` | `AppearanceSettings`, `ThemeState`, `LightDMSettings`, `IThemeRepository`, `IWallpaperRepository`, `ILightDMRepository` |

---

### 2.2 `src/application/` (Use Cases)
Implements single-purpose business use cases orchestrating domain repositories:

- **`application/audio/adjust_volume_use_case.py`**: Controls audio output volume and mute toggles.
- **`application/clipboard/manage_clipboard_use_case.py`**: Interacts with clipboard history, pinning favorites, clearing history.
- **`application/display/set_brightness_use_case.py`**: Adjusts backlight brightness.
- **`application/display/switch_display_mode_use_case.py`**: Swaps monitor modes (PC Only, Extend, Duplicate, External Only).
- **`application/media/take_screenshot_use_case.py`**: Captures screen regions or full monitors.
- **`application/menu/show_menu_use_case.py`**: Launches app menu or filters categories.
- **`application/notification/send_notification_use_case.py`**: Dispatches desktop notifications.
- **`application/power/lock_screen_use_case.py`**: Triggers screen lock (`swaylock`).
- **`application/power/toggle_battery_conservation_use_case.py`**: Toggles Lenovo IdeaPad battery charge threshold (80% vs 100%).
- **`application/power/toggle_idle_use_case.py`**: Toggles automatic system suspension (`swayidle`).
- **`application/power/toggle_power_profile_use_case.py`**: Cycles CPU power profiles (`power-saver`, `balanced`, `performance`).
- **`application/theme/manage_appearance_use_case.py`**: Reads/writes GTK/Qt appearance themes.
- **`application/theme/set_wallpaper_use_case.py`**: Changes Sway background wallpapers.
- **`application/theme/toggle_theme_use_case.py`**: Switches between dark and light themes.
- **`application/theme/update_lightdm_use_case.py`**: Configures LightDM GTK greeter.

---

### 2.3 `src/infrastructure/` (Adapters & External Systems)
Contains technology implementations interacting with Wayland compositors, D-Bus, Sysfs, binaries, and sockets:

- **`infrastructure/audio/mixer_audio_repository.py`**: `pactl`/`wpctl`/`pamixer` wrapper.
- **`infrastructure/clipboard/cliphist_repository.py`**: `cliphist` history binary and file-backed favorites storage (`~/.config/sway-manager/favorites.json`).
- **`infrastructure/config/json_config_repository.py`**: File persistence for settings at `~/.config/sway-manager/config.json`.
- **`infrastructure/daemon/`**:
  - `daemon_server.py`: `SwayManagerDaemon` managing `QLocalServer` Unix socket listening and GC.
  - `daemon_client.py`: `SwayManagerClient` sending JSON commands over socket.
  - `daemon_utils.py`: Executable path resolution for compiled Nuitka binaries vs development Python scripts.
  - `server/cli.py`: Synchronous command handler (`CliServerHandler`).
  - `server/gui.py`: Subprocess launcher for GUI commands (`GuiServerHandler`).
  - `server/config/gui_config.py`: `setup_qt_environment()` resilience flags and cache initialization.
  - `server/config/cli_config.py`: Exception logging handlers.
- **`infrastructure/display/`**:
  - `brightnessctl_repository.py`: `brightnessctl` wrapper.
  - `sway_display_repository.py`: `swaymsg -t get_outputs` parser and layout switcher.
- **`infrastructure/logging/async_logger.py`**: Thread-safe non-blocking file logger writing to `~/.config/sway-manager/logs/log-YYYY-MM-DD.txt`.
- **`infrastructure/media/grim_slurp_screenshot_repository.py`**: Captures screenshots using `grim` and `slurp`.
- **`infrastructure/menu/`**:
  - `desktop_parser.py`: Scans `/usr/share/applications` for `.desktop` files.
  - `icon_resolver.py`: Maps XDG icon names to image file paths.
  - `wofi_launcher.py`: Invokes `wofi` launcher.
- **`infrastructure/notification/desktop_notification_repository.py`**: `notify-send` wrapper.
- **`infrastructure/power/`**:
  - `sysfs_battery_repository.py`: Reads/writes `/sys/bus/platform/drivers/ideapad_acpi/VPC2004:00/conservation_mode`.
  - `swayidle_repository.py`: Manages `swayidle` daemon processes.
  - `swaylock_repository.py`: Launches `swaylock`.
  - `powerprofiles_repository.py`: `powerprofilesctl` or `power-profiles-daemon` D-Bus adapter.
- **`infrastructure/theme/`**:
  - `gtk_qt_theme_repository.py`: Updates `~/.config/gtk-3.0/settings.ini` and `~/.config/qt5ct/qt5ct.conf`.
  - `lightdm_repository.py`: Reads/writes `/etc/lightdm/lightdm-gtk-greeter.conf`.
  - `sway_wallpaper_repository.py`: Sets wallpapers using `swaymsg output * bg <file> fill`.

---

### 2.4 `src/presentation/` (UI & Router)
- **`presentation/cli/router.py`**: Standard entrypoint router handling `--standalone`, `daemon`, `portal`, and argument dispatching.
- **`presentation/cli/handlers.py`**: Maps CLI command strings to use cases and PySide6 windows.
- **`presentation/gui/`**:
  - `app_factory.py`: `ApplicationFactory.buildWidget` managing Qt application loop and memory cleanup.
  - `styles.py`: QSS stylesheet tokens (Dark/Light mode).
  - `windows/config_center.py`: `ConfigCenterWindow` (Settings panel).
  - `windows/monitor_swap.py`: `MonitorSwapWindow` (Display switcher).
  - `windows/wallpaper_picker.py`: `WallpaperPickerWindow` (Visual wallpaper picker).
  - `windows/brightness_popup.py`: `BrightnessPopup` (Slider popup).
  - `osd/brightness_osd.py`, `osd/volume_osd.py`: Transient On-Screen Displays.
  - `widgets/freeze_selection_overlay.py`: `FreezeSelectionOverlay` for interactive area screenshot cropping.

---

### 2.5 `src/portal/` (XDG Desktop Portal Screencast Chooser)
- **`controller.py`**: `PortalController` orchestrating screen and window source detection.
- **`diagnostics.py`**: `PortalDiagnostics` reporting session support status.
- **`outputs_provider.py`**: `SwayOutputsProvider` parsing active outputs from `swaymsg`.
- **`windows_provider.py`**: `WindowSharingProvider` inspecting open windows via `lswt`.
- **`result_writer.py`**: `PortalResultWriter` outputting single selection lines to `stdout`.
- **`config_installer.py`**: `PortalConfigInstaller` configuring `xdg-desktop-portal-wlr` INI files.
- **`dialog.py`**: `PortalDialog` PySide6 window for selecting monitors/windows.

---

## 3. Entry Points & Invocation Maps

### `src/main.py`
```python
from presentation.cli.router import run_cli

if __name__ == "__main__":
    run_cli()
```

### `src/gui_main.py`
```python
import sys
from presentation.cli.handlers import CLIHandlers

def run_gui_app():
    args = list(sys.argv)
    cmd = str(args[1]).lower()
    if cmd in ("settings", "config", "config-center"):
        CLIHandlers.handle_settings()
    elif cmd == "monitor":
        CLIHandlers.handle_monitor(args)
    # ...
```

---

## 4. Build & Distribution Structure

- **Compilation**: `./build.sh` invokes Nuitka to compile standalone binaries `SwayManager` and `SwayManagerGUI` into `out/main.dist/`.
- **Symlinks**: Creates symlinks `sway-manager` and `sway-manager-gui` inside `out/main.dist/`.
- **Installation**: `./install.sh` copies contents of `out/main.dist/` to `$HOME/.config/sway/bin/sway-manager/` and creates global symlinks in `$HOME/.config/sway/bin/`.
