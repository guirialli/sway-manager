# SwayManager — Specs & Long-Term System Memory

This directory contains the authoritative, long-term technical specifications for **SwayManager**. It serves as the primary architectural memory for developers and AI agents working on this codebase.

All specifications herein are grounded directly in the implementation source code (`src/`), build scripts (`build.sh`, `install.sh`), and unit tests (`tests/`).

---

## 1. Specification Index

### Architecture & Infrastructure
- **[`architecture.md`](./architecture.md)**: System overview, Clean Architecture / DDD-light design, component interactions, and execution models.
- **[`project-structure.md`](./project-structure.md)**: Repository layout, layer responsibilities, file organization, build and installation workflows.
- **[`daemon.md`](./daemon.md)**: `SwayManagerDaemon` architecture, Qt event loop integration, socket listening, process lifecycle, and memory management.
- **[`client-server.md`](./client-server.md)**: Unix Domain Socket IPC protocol specification, request/response JSON schemas, connection lifecycle, and execution routing.
- **[`sway-integration.md`](./sway-integration.md)**: Sway/SwayFX compositor IPC, `swaymsg` invocations, and external Wayland tool dependencies (`grim`, `slurp`, `lswt`, `wofi`, `cliphist`, `swaylock`, `swayidle`, `brightnessctl`).
- **[`portal.md`](./portal.md)**: `xdg-desktop-portal-wlr` screen sharing chooser contract, stdout result writer protocol, portal diagnostics, and configuration installer.

### Application Interfaces & Execution
- **[`cli.md`](./cli.md)**: CLI router (`src/presentation/cli/router.py`), command dispatcher, `--standalone` execution flag, and stdout/stderr contracts.
- **[`gui.md`](./gui.md)**: PySide6 GUI architecture (`src/gui_main.py`), `ApplicationFactory` widget management, standalone process isolation, Qt Wayland environment setup, and window/OSD components.
- **[`theme-design.md`](./theme-design.md)**: Cyberpunk visual design system, black and neon purple color tokens (`DARK_COLORS`), component styling, and QPalette mapping.
### Operations & Conventions
- **[`configuration.md`](./configuration.md)**: System configuration files (`~/.config/sway-manager/config.json`), environment variables, LightDM greeter settings, and udev rules.
- **[`testing.md`](./testing.md)**: `unittest` test suite structure, running tests headlessly, mocking strategies for Wayland/subprocess binaries, and test coverage catalog.
- **[`conventions.md`](./conventions.md)**: Python coding standards, typing guidelines, exception handling hierarchies (`SwayException`), logging (`AsyncLogger`), and Qt memory management.

### Architectural Decision Records (`specs/adrs/`)
- **[`adrs/0001-xdg-portal-integration.md`](./adrs/0001-xdg-portal-integration.md)**: XDG Desktop Portal integration & simple chooser strategy.
- **[`adrs/0002-daemon-client-ipc-model.md`](./adrs/0002-daemon-client-ipc-model.md)**: Daemon-client Unix Domain Socket IPC & execution model.
- **[`adrs/0004-polkit-agent.md`](./adrs/0004-polkit-agent.md)**: Native polkit agent (zbus backend + Tauri dialog).
### Subsystem Domains (`specs/domains/`)
- **[`domains/media.md`](./domains/media.md)**: Screenshots (full, area, window) using `grim`, `slurp`, and the HiDPI freeze overlay.
- **[`domains/display.md`](./domains/display.md)**: Monitor switching (PC only, duplicate, extend, monitor only) and backlight brightness control.
- **[`domains/power.md`](./domains/power.md)**: Battery conservation mode (`sysfs`), idle inhibition (`swayidle`), screen lock (`swaylock`), and power profiles (`power-profiles-daemon`).
- **[`domains/clipboard.md`](./domains/clipboard.md)**: Clipboard history management, favorite items pinning/unpinning, and `cliphist` integration.
- **[`domains/menu.md`](./domains/menu.md)**: Application launcher, `.desktop` entry parser, icon resolution, and Wofi menu integration.
- **[`domains/theme.md`](./domains/theme.md)**: GTK/Qt dark/light mode synchronization, Sway wallpaper management, and LightDM greeter configuration.
- **[`domains/audio.md`](./domains/audio.md)**: Audio volume management, mute toggling, and Waybar status indicators.
- **[`domains/polkit.md`](./domains/polkit.md)**: Native polkit authentication agent (D-Bus backend + Tauri auth dialog).

---

## 2. Reading Guide for AI Agents & Developers

Before undertaking modifications, consult the appropriate specification files based on your task target:

| Task Type | Mandatory Specs to Read First |
|---|---|
| Modifying the Daemon or IPC | `architecture.md`, `daemon.md`, `client-server.md` |
| Adding or modifying CLI commands | `cli.md`, `client-server.md` |
| Creating or updating PySide6 GUI windows/popups | `gui.md`, `conventions.md` |
| Integrating Sway IPC or Wayland CLI tools | `sway-integration.md`, relevant `domains/*.md` |
| Working on Screen Sharing / XDG Desktop Portal | `portal.md`, `gui.md` |
| Writing or fixing unit tests | `testing.md`, `conventions.md` |
