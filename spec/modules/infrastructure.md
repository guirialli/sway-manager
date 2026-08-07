# Module Spec: Infrastructure Layer (`src/infrastructure/`)

## 1. Module Overview
Provides concrete implementations of domain interfaces by encapsulating external tools, Sway IPC, Linux sysfs filesystem interactions, D-Bus services, and background daemon Unix socket communication.

---

## 2. Infrastructure Packages

### 2.1 Display & Audio Repositories (`infrastructure/display/`, `infrastructure/audio/`)
- `SwayDisplayRepository`: Executes `swaymsg -t get_outputs`, parses output configurations, manages `wl-mirror` duplication processes.
- `BrightnessctlRepository`: Communicates with `brightnessctl` binary to read/set backlight level.
- `MixerAudioRepository`: Interacts with ALSA `amixer` or PipeWire `wpctl` to read/set master audio volume and mute state.

### 2.2 Power Repositories (`infrastructure/power/`)
- `SysfsBatteryRepository`: Reads/writes `/sys/bus/platform/drivers/ideapad_acpi/.../charge_control_end_threshold` or vendor battery conservation node.
- `SwayIdleRepository`: Manages `swayidle` process state, toggling suspension inhibitors.
- `PowerProfilesRepository`: Communicates with `power-profiles-daemon` over system D-Bus.
- `SwayLockRepository`: Builds swaylock configuration files based on theme tokens and triggers `swaylock`.

### 2.3 Media, Menu & Clipboard (`infrastructure/media/`, `infrastructure/menu/`, `infrastructure/clipboard/`)
- `GrimSlurpScreenshotRepository`: Invokes `grim` and `slurp` for interactive full/area/window screenshots and copies image payload to `wl-copy`.
- `WofiRepository`: Builds dynamic Wofi menu lists for app categories.
- `CliphistRepository`: Interacts with `cliphist list`, `cliphist decode`, and manages pinned favorites at `~/.config/sway-manager/clipboard_favorites.json`.

### 2.4 Daemon IPC & Logging (`infrastructure/daemon/`, `infrastructure/logging/`)
- `SwayManagerDaemon`: Async socket server listening on `~/.config/sway-manager/daemon.sock`.
- `SwayManagerClient`: CLI IPC client that dispatches commands to daemon or spawns GUI executables (`SwayManagerGUI`).
- `AsyncLogger`: Thread-isolated logger using an `asyncio` event loop to flush log entries to disk asynchronously.

---

## 3. Subprocess Safety Invariants
1. All subprocess executions must specify `capture_output=True` and `text=True` or check returncodes cleanly.
2. Binary paths must be validated or fail gracefully with clear `SwayException` exceptions.
3. System calls must not pollute `stdout` when invoked during portal chooser operations.
