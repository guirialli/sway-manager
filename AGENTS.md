# Repository Guidelines

## Project Structure & Module Organization

`src/` contains the Python application, organized by responsibility: `domain/`
defines entities and repository interfaces, `application/` contains use cases,
`infrastructure/` integrates Sway, system tools, and persistence, and
`presentation/` contains CLI handlers and PySide6 widgets/windows. Entry points
are `src/main.py` (daemon/CLI) and `src/gui_main.py` (interactive GUI commands).
Put unit tests in `tests/test_<feature>.py`. Runtime dependencies are declared in
`requirements.txt`; `udev/` holds the battery rule.

## Build, Test, and Development Commands

- `python3 -m venv venv && source venv/bin/activate` creates a local environment.
- `pip install -r requirements.txt` installs PySide6 and runtime dependencies.
- `PYTHONPATH=src python3 -m unittest discover -s tests -v` runs the test suite.
- `python3 src/main.py --help` exercises the CLI entry point.
- `./build.sh` runs tests and produces standalone Nuitka binaries in `out/`.
- `./install.sh --no-udev` installs the build locally without changing system
  udev rules. Do not run the default installer unless system installation is
  intended.

## Coding Style & Naming Conventions

Use Python 3.10+ with four-space indentation, type hints on new public code,
and standard import grouping (stdlib, third party, local). Use `snake_case` for
functions, variables, and modules; `PascalCase` for classes; and
`UPPER_CASE` for constants. Keep UI code in `presentation/gui/`; keep Sway,
`grim`, `slurp`, and filesystem calls behind infrastructure repositories. Prefer
small explicit methods and specific exception handling over broad `except`.

## Testing Guidelines

Tests use `unittest` and `unittest.mock`. Name test files `test_*.py`, test
classes `Test<Subject>`, and test methods `test_<behavior>`. Mock external
commands, sockets, desktop services, and system paths; do not require a live
Sway session. For Qt image/widget tests, run headlessly with
`QT_QPA_PLATFORM=offscreen`.

## Commit & Pull Request Guidelines

History uses Conventional Commit-style prefixes, primarily `feat:` and `fix:`;
write concise imperative summaries, for example `fix: preserve HiDPI screenshot
selection`. Keep commits focused. Pull requests should describe user-visible
behavior, list validation commands, link related issues when available, and add
screenshots or a short recording for GUI changes. Call out Sway, Wayland, or
system-level prerequisites explicitly.
