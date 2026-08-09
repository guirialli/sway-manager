# SwayManager — Conventions Specification

## 1. Python Code Style & Standards

SwayManager target environment is **Python 3.10+** (tested up to 3.13).

- **Formatting**: PEP 8 compliance, 4-space indentation, 100-character max line length.
- **Naming Conventions**:
  - `snake_case` for module names, variables, functions, and method names.
  - `PascalCase` for class names (`SwayManagerDaemon`, `PortalController`, `SetBrightnessUseCase`).
  - `UPPER_CASE` for constants (`GUI_COMMANDS`, `INTERACTIVE_COMMANDS`, `PORTALS_CONF_SECTION`).
- **Imports**: Grouped strictly in standard sequence separated by blank lines:
  1. Python Standard Library (`os`, `sys`, `json`, `subprocess`, `typing`).
  2. Third-Party Libraries (`PySide6.QtCore`, `PySide6.QtWidgets`, `PySide6.QtGui`).
  3. Internal Local Modules (`domain.*`, `application.*`, `infrastructure.*`, `presentation.*`, `portal.*`).

---

## 2. Type Hints & Dataclasses

1. **Type Hints**: All new or modified public functions, methods, and dataclass attributes MUST include explicit Python type annotations (`str`, `int`, `bool`, `list[str]`, `dict[str, Any]`, `Optional[T]`).
2. **Immutable Dataclasses**: Domain entities and value objects SHOULD use `@dataclass(frozen=True)` to enforce immutability:
   ```python
   @dataclass(frozen=True)
   class BatteryState:
       is_supported: bool
       is_conservation_on: bool
       raw_value: int
       control_file: Optional[str] = None
   ```

---

## 3. Exception Handling Hierarchy

1. **Specific Exceptions**: Avoid broad `except Exception:` unless in top-level crash-protection boundaries (`_safe_dispatch_command`). Catch explicit system errors (`FileNotFoundError`, `subprocess.CalledProcessError`, `json.JSONDecodeError`).
2. **Domain Exception Base**: Inherit custom exceptions from domain base exceptions defined in `utils/exceptions.py` or domain files:
   ```python
   class SwayException(Exception):
       """Base exception for SwayManager domain errors."""
       pass

   class SwayNotAvailableError(PortalException):
       """Raised when Sway compositor or IPC socket is unreachable."""
       pass
   ```
3. **No Silent Swallowing**: Exceptions caught at infrastructure boundaries MUST be logged via `AsyncLogger.error()` before returning fallback states.

---

## 4. Logging & Diagnostics (`AsyncLogger`)

1. **Stdout Cleanliness**: `print()` to `stdout` is reserved strictly for CLI command outputs (e.g. Waybar status JSON, portal selection lines).
2. **Async Logger**: Application logs MUST be routed through `get_logger()` (`AsyncLogger` singleton):
   ```python
   from infrastructure.logging.async_logger import get_logger

   logger = get_logger()
   logger.info("Service initialized.")
   logger.cmd("Executing command: wallpaper")
   logger.error("Failed to execute swaymsg", exc_info=True)
   ```

---

## 5. Qt / PySide6 Guidelines

1. **Thread Safety**: Never interact with Qt UI widgets from background Python threads. Use `QTimer.singleShot` or Qt Signals/Slots to marshal execution back to the main thread.
2. **Memory Ownership**: Widgets MUST set `Qt.WidgetAttribute.WA_DeleteOnClose` to `True` to ensure automatic memory release when closed.
3. **Memory Cleanup Invariant**: Every custom window SHOULD connect its `destroyed` signal to `_cleanup_memory()` to clear `QPixmapCache` and run `malloc_trim(0)`.
4. **Wayland Flags**: Always call `setup_qt_environment()` before instantiating `QApplication`.
