# Module Spec: Domain Layer (`src/domain/`)

## 1. Module Overview
The Domain layer defines core entities, value objects, dataclasses, enums, and repository interfaces. It contains zero third-party UI or framework imports and acts as the technical contract for the application layer.

---

## 2. Subpackages & Entities

### 2.1 Display Domain (`src/domain/display/`)
- **Entities / Value Objects**:
  - `DisplaySwitchType(Enum)`: `PC_ONLY ("0")`, `DUPLICATE ("1")`, `EXTEND ("2")`, `MONITOR_ONLY ("3")`.
  - `MonitoresSway(dataclass)`: `interno: str`, `externo: str`.
- **Repository Interface (`IDisplayRepository`)**:
  ```python
  class IDisplayRepository(ABC):
      @abstractmethod
      def get_monitors(self) -> MonitoresSway: ...
      @abstractmethod
      def apply_config(self, mode: DisplaySwitchType) -> None: ...
      @abstractmethod
      def recarregar_sway(self) -> None: ...
      @abstractmethod
      def get_connected_monitors_count(self) -> int: ...
      @abstractmethod
      def get_current_layout(self) -> DisplaySwitchType: ...
  ```

### 2.2 Power Domain (`src/domain/power/`)
- **Entities / Value Objects**:
  - `BatteryState(dataclass)`: `supported: bool`, `enabled: bool`, `percentage: int`.
  - `IdleState(dataclass)`: `inhibited: bool`, `pid: int | None`.
  - `PowerProfileState(dataclass)`: `profile: str` (`performance`, `balanced`, `power-saver`).
- **Repository Interfaces**: `IBatteryRepository`, `IIdleRepository`, `IPowerProfilesRepository`.

### 2.3 Theme & Media Domains (`src/domain/theme/`, `src/domain/media/`, `src/domain/clipboard/`)
- **Entities**: `LightDMSettings`, `AppearanceSettings`, `ClipboardItem`, `ScreenshotMode`.
- **Interfaces**: `IThemeRepository`, `IWallpaperRepository`, `IScreenshotRepository`, `IClipboardRepository`.

---

## 3. Invariants & Principles
1. **Framework Independence**: No Qt (`PySide6`), `subprocess`, or `sysfs` calls allowed in `domain/`.
2. **Immutability**: Value objects and dataclasses use `frozen=True` where appropriate.
3. **Explicit Typing**: All abstract methods require complete type hints.
