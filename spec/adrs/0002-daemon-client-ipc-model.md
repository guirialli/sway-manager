# ADR 0002: Daemon-Client Socket IPC & Execution Model

- **Status**: Accepted
- **Date**: 2026-08-04
- **Authors**: SwayManager Core Team

## Context & Problem Statement
SwayManager manages various system controls (battery thresholds, idle inhibitors, power profiles, brightness, volume, wallpaper). Repeatedly starting a full Python interpreter and initializing heavy libraries for instant keybindings (e.g. brightness up/down) introduces latency and CPU overhead.

We needed a fast, persistent execution model while supporting standalone GUI window launches.

## Decision Drivers
1. **Low Latency**: Keybindings and status updates must execute in under 50ms.
2. **Resource Efficiency**: Single long-running daemon process maintaining warm connections and system state.
3. **Decoupled Architecture**: CLI client (`SwayManager`) can communicate with the daemon via Unix Domain Socket or fall back to standalone execution if the daemon is stopped.
4. **GUI Separation**: Interactive GUI tools (`SwayManagerGUI`, `config-center`, `monitor`) should run as dedicated processes when triggered by the daemon to keep the daemon lightweight and responsive.

## Decision Outcome
We implemented an Async Unix Domain Socket daemon server (`SwayManagerDaemon`) listening at `~/.config/sway-manager/daemon.sock`.

### Execution Flow:
1. `SwayManager daemon` starts the persistent daemon process in background.
2. `SwayManager <command>` acts as a lightweight client:
   - Connects to the socket.
   - Sends JSON-encoded command arguments.
   - For background tasks (e.g. `battery toggle`, `osd volume up`), the daemon executes the use case and returns the output to the client.
   - For GUI tasks (e.g. `settings`, `monitor`), the daemon spawns `SwayManagerGUI` as a separate process.
3. `SwayManager portal` bypasses the daemon client to run directly locally as an `xdg-desktop-portal-wlr` subprocess.

## Consequences

### Positive
- Sub-50ms execution time for keybindings and Waybar status calls.
- Centralized state management without race conditions.
- Standalone fallback capability when daemon is inactive.

### Trade-offs / Limitations
- Requires managing socket file lifecycle and stale socket cleanup.
