# ADR 0001: XDG Desktop Portal Integration & Simple Chooser Strategy

- **Status**: Accepted
- **Date**: 2026-08-06
- **Authors**: SwayManager Core Team

## Context & Problem Statement
Applications running under Wayland (browsers, video conferencing software, recording applications like OBS) require screen sharing through PipeWire using the XDG Desktop Portal system. `xdg-desktop-portal-wlr` handles PipeWire stream creation on Sway/wlroots compositors but requires an external command/dialog to allow the user to choose which screen or window to share.

We needed an integrated visual selector matching SwayManager's PySide6 Apple HIG styling that strictly respects the line-based stdout contract expected by `xdg-desktop-portal-wlr`.

## Decision Drivers
1. **User Experience**: Consistent, beautiful GUI dialog matching the rest of SwayManager.
2. **Protocol Safety**: `xdg-desktop-portal-wlr` expects exact line protocol on stdout (`Monitor: <name>` or `Window: <id>`). Diagnostic logs must never leak into stdout.
3. **Graceful Fallback**: Support full monitor sharing on all Sway versions, while allowing window capture when Sway $\ge$ 1.12 and `lswt` are present.
4. **Non-Intrusive Installation**: Automate `portals.conf` and `wlr/config` setup while preserving user backups.

## Decision Outcome
We implemented a dedicated sub-package `src/portal/` and CLI command `SwayManager portal` configured as `chooser_type=simple` in `~/.config/xdg-desktop-portal-wlr/config`.

### Architecture Choices:
- `PortalResultWriter`: Encapsulates all stdout writes, ensuring single-line output and automatic flush.
- `Lazy Module Imports`: `PortalController` is imported lazily in `src/portal/__init__.py` so non-GUI tools (like `PortalConfigInstaller`) can execute without requiring PySide6 dependencies.
- `Bypass Daemon`: The `portal` command bypasses the daemon client IPC to execute locally as a chooser subprocess directly spawned by `xdg-desktop-portal-wlr`.

## Consequences

### Positive
- Full integration with Wayland screen sharing using native PySide6 widgets.
- Clean separation of diagnostic logs (`stderr`) and portal contracts (`stdout`).
- Automated configuration setup with zero-loss backup.

### Trade-offs / Limitations
- Window sharing depends on Sway 1.12+ (ext-foreign-toplevel-list-v1) and `lswt`; older Sway versions fall back to monitor-only selection.
