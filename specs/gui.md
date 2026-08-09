# SwayManager — GUI Specification

## 1. Overview & Architecture

SwayManager's graphical user interfaces are built with **PySide6 (Qt6 for Python)**. To maintain desktop performance and avoid persistent Wayland connection leaks, all GUI modules are designed as standalone, self-contained widgets or windows managed by `ApplicationFactory` (`src/presentation/gui/app_factory.py`).

GUI entry point: `src/gui_main.py` (executes binary `SwayManagerGUI`).

```mermaid
flowchart TD
    Launcher[SwayManagerGUI <cmd>] --> RouteCmd{Cmd?}
    RouteCmd -- settings/config --> CC[ConfigCenterWindow]
    RouteCmd -- monitor --> MS[MonitorSwapWindow]
    RouteCmd -- wallpaper --> WP[WallpaperPickerWindow]
    RouteCmd -- osd/brilho --> BP[BrightnessPopup / BrightnessOSD / VolumeOSD]
    RouteCmd -- screenshot --> FSO[FreezeSelectionOverlay]

    CC --> AppFactory[ApplicationFactory.buildWidget]
    MS --> AppFactory
    WP --> AppFactory
    BP --> AppFactory
    FSO --> AppFactory

    AppFactory --> SetDesktopName[QApplication.setDesktopFileName]
    AppFactory --> ShowWidget[widget.show raise activateWindow]
    AppFactory --> ConnectCleanup[widget.destroyed.connect cleanup_memory]
    AppFactory --> ExecApp[app.exec()]
```

---

## 2. Qt Wayland Resilience & Environment Setup

Before any Qt module is imported or `QApplication` instantiated, `setup_qt_environment()` (`src/infrastructure/daemon/server/config/gui_config.py`) configures environment variables to guarantee compatibility with Wayland compositors:

```python
def setup_qt_environment():
    os.environ["QT_QPA_PLATFORM"] = "wayland;xcb"
    os.environ["QT_WAYLAND_DISABLE_WINDOWDECORATION"] = "1"
    os.environ["QT_LOGGING_RULES"] = "*.debug=false;qt.qpa.*=false"
```

### Purpose:
- `QT_QPA_PLATFORM="wayland;xcb"`: Prefers native Wayland backend with XCB fallback.
- `QT_WAYLAND_DISABLE_WINDOWDECORATION="1"`: Prevents Qt from drawing redundant server-side titlebars under Sway, allowing Sway to manage window borders.
- `QT_LOGGING_RULES`: Suppresses debug log spam from Qt QPA in `stderr`.

---

## 3. Widget Lifecycle & Memory Management (`ApplicationFactory`)

`ApplicationFactory` manages single-instance or standalone Qt widget rendering, ensuring aggressive memory reclamation on window close.

```python
class ApplicationFactory:
    _active_widgets: list[QWidget] = []

    @classmethod
    def buildWidget(
        cls,
        fn_create_widget: Callable[[], QWidget],
        desktop_file_name: str | None = None,
    ):
        app = QApplication.instance()
        is_standalone = app is None

        if is_standalone:
            app = QApplication(sys.argv)

        if desktop_file_name:
            QApplication.setDesktopFileName(desktop_file_name)

        try:
            widget = fn_create_widget()
            widget.setAttribute(Qt.WidgetAttribute.WA_DeleteOnClose, True)
            cls._active_widgets.append(widget)

            widget.destroyed.connect(
                lambda: _cleanup_memory(widget, cls._active_widgets)
            )

            widget.show()
            widget.raise_()
            widget.activateWindow()
        except Exception as ex:
            # Log error
            if is_standalone:
                sys.exit(1)
            return

        if is_standalone:
            sys.exit(app.exec())
```

### Memory Cleanup Invariants (`_cleanup_memory`):
When a widget emits `destroyed`:
1. It is removed from `_active_widgets`.
2. `QPixmapCache.clear()` flushes cached image surfaces.
3. `gc.collect()` reclaims Python objects.
4. `ctypes.CDLL("libc.so.6").malloc_trim(0)` instructs glibc to release freed heap memory pages back to Linux.

---

## 4. Main GUI Windows Catalog

### 4.1 Config Center (`ConfigCenterWindow`)
- Path: `src/presentation/gui/windows/config_center.py`
- Command: `SwayManagerGUI settings` or `SwayManagerGUI config-center`
- Description: Central control panel with tabs for GTK/Qt appearance themes, LightDM greeter settings, display profiles, and system settings.

### 4.2 Monitor Swap Window (`MonitorSwapWindow`)
- Path: `src/presentation/gui/windows/monitor_swap.py`
- Command: `SwayManagerGUI monitor`
- Desktop File Name: `sway.apps.monitor-swap`
- Description: Graphical display layout switcher allowing one-click toggling between:
  - **PC Only**: Disables external monitor, enables internal laptop screen.
  - **Duplicate**: Clones laptop screen to external output.
  - **Extend**: Extends workspace across monitors.
  - **Monitor Only**: Disables laptop screen, enables external monitor.

### 4.3 Wallpaper Picker Window (`WallpaperPickerWindow`)
- Path: `src/presentation/gui/windows/wallpaper_picker.py`
- Command: `SwayManagerGUI wallpaper [directory_path]`
- Description: Visual grid picker parsing image files (`.png`, `.jpg`, `.jpeg`, `.webp`) from wallpaper directories, allowing live preview and background setting via Sway IPC.

### 4.4 Brightness Popup Window (`BrightnessPopup`)
- Path: `src/presentation/gui/windows/brightness_popup.py`
- Command: `SwayManagerGUI brightness popup` or `SwayManagerGUI osd brilho popup`
- Description: Interactive popup slider for manual display brightness adjustment.

---

## 5. On-Screen Display (OSD) Components

- Paths: `src/presentation/gui/osd/brightness_osd.py`, `src/presentation/gui/osd/volume_osd.py`
- Base Class: `BaseOSD` (`src/presentation/gui/components/base_osd.py`)
- Window Flags: `Qt.WindowStaysOnTopHint | Qt.FramelessWindowHint | Qt.Tool | Qt.WindowTransparentForInput`
- Behavior: Displays transient HUD overlay displaying current brightness or audio volume percentage. Auto-closes after a 1.5-second `QTimer` countdown.

---

## 6. Freeze Selection Overlay (`FreezeSelectionOverlay`)

- Path: `src/presentation/gui/widgets/freeze_selection_overlay.py`
- Command Trigger: `SwayManager screenshot area`
- Description: High-performance screenshot selection tool.
  - Takes a full screenshot of all active monitors with `grim`.
  - Maps physical monitor pixel coordinates and HiDPI scales.
  - Renders a full-screen semi-transparent dark overlay.
  - Allows user mouse drag to draw selection rectangle.
  - Emits selected geometric bounds `(x, y, width, height)` or physical crop image upon mouse release.
