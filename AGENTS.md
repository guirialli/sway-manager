# Repository Guidelines

## Project Structure & Module Organization

`daemon/` is the Rust workspace root (`sway-manager-daemon` lib + `SwayManager`
binary): `src/repos/` holds one module per system domain (audio, portal,
screenshot, wallpaper…), `src/handlers.rs` routes CLI argv, `src/server.rs`
serves the Unix-socket IPC, `src/main.rs` wires CLI/daemon startup.
`gui/src-tauri/` is the Tauri v2 GUI (`SwayManagerGUI` binary): `src/main.rs`
routes one `WebviewWindow` per command, `src/tauri_runtime.rs` exposes
`#[tauri::command]` wrappers over daemon repos, `capabilities/main.json`
grants frontend permissions. `gui/src/routes/*.html` are the vanilla-JS
frontends (no framework). `udev/` holds the battery rule.

## Build, Test, and Development Commands

- `cargo test` (in `daemon/`) runs the daemon test suite (lib + bin + integration).
- `cargo test` (in `gui/src-tauri/`) runs the GUI test suite.
- `cargo clippy --all-targets` must be warning-free in both crates.
- `./build.sh` runs both suites and produces `out/main.dist/` with the Rust
  `SwayManager` daemon and Tauri `SwayManagerGUI` GUI (`sway-manager-gui` stays
  as install alias). It `touch`es `tauri.conf.json` so HTML/CSS/JS is re-embedded.
- `./install.sh --no-udev` installs the build locally without changing system
  udev rules. Do not run the default installer unless system installation is
  intended.

## Coding Style & Naming Conventions

Use Rust 2024 edition, `cargo fmt` defaults. Use `snake_case` for functions,
variables, and modules; `PascalCase` for types; `UPPER_CASE` for constants.
Keep Sway, `wpctl`, `grim`, `slurp`, and filesystem calls behind `repos/`
modules behind the `CommandRunner` seam (injectable `FakeRunner` in tests).
Prefer small explicit functions, `Result`/`Option` over panics (no `unwrap`
outside tests), and specific error values over generic strings.

## Testing Guidelines

Tests use Rust built-in `#[test]`/`#[tokio::test]` plus `tempfile` for
filesystem isolation. Name integration files `tests/<feature>.rs`, test fns
`test_<behavior>` or descriptive snake_case. Mock external commands via
`FakeRunner`, sockets via tempdir Unix listeners, and never require a live
Sway session. Every new repo/handler/command needs a regression test;
socket-visible behavior needs a socket round-trip test.

## Commit & Pull Request Guidelines

History uses Conventional Commit-style prefixes, primarily `feat:` and `fix:`;
write concise imperative summaries, for example `fix: preserve HiDPI screenshot
selection`. Keep commits focused. Pull requests should describe user-visible
behavior, list validation commands, link related issues when available, and add
screenshots or a short recording for GUI changes. Call out Sway, Wayland, or
system-level prerequisites explicitly.
