# dotfile-rs-tui

A small Rust terminal UI (TUI) application to manage dotfiles (UI / CLI front-end).

## Table of contents
- Project structure
- Prerequisites
- IDE / Editor
- Build
- Run
- Debug (VS Code)
- Debug (RustRover)
- Notes

## Project structure
- Cargo.toml
- src/
    - main.rs
    - (other modules: e.g. ui.rs, app.rs, etc.)
- assets/ (optional: templates, config)
- README.md
- .vscode/ (optional: launch/tasks)

## Prerequisites
- Rust toolchain via rustup (stable channel). Ensure cargo is in PATH.
- Windows (instructions assume PowerShell or VS Code integrated terminal).
- Optional: rustfmt and clippy components:
  - rustup component add rustfmt clippy

## IDE / Editor
- Visual Studio Code (recommended extensions)
  - rust-analyzer
  - CodeLLDB
- JetBrains RustRover (alternative IDE with built-in run/debug support for Cargo)

## Build
Open PowerShell or the VS Code integrated terminal at the repository root:
- Debug build:
    cargo build
- Release build:
    cargo build --release

## Run
- Run in debug mode:
    cargo run
- Run release build:
    cargo run --release
- Run tests:
    cargo test
- Format and lint:
    cargo fmt
    cargo clippy

## Debug (VS Code)
1. Install rust-analyzer and CodeLLDB.
2. Add .vscode/launch.json and .vscode/tasks.json (examples below). Update the program path if your binary name differs.

Example .vscode/launch.json (place in .vscode/launch.json):
```json5
    {
        "version": "0.2.0",
        "configurations": [
            {
                "type": "lldb",
                "request": "launch",
                "name": "Debug (cargo build then run)",
                "preLaunchTask": "cargo build",
                "program": "${workspaceFolder}/target/debug/dotfile-rs-tui",
                "args": [],
                "cwd": "${workspaceFolder}",
                "console": "integratedTerminal"
            }
        ]
    }

```

Example .vscode/tasks.json (place in .vscode/tasks.json):
```json5
    {
        "version": "2.0.0",
        "tasks": [
            {
                "label": "cargo build",
                "type": "shell",
                "command": "cargo build",
                "group": {
                    "kind": "build",
                    "isDefault": true
                },
                "problemMatcher": ["$rustc"]
            }
        ]
    }
```
Note: TUI apps may render incorrectly in some integrated terminals. If you see UI issues, set "console" to "externalTerminal" in launch.json.

## Debug (RustRover)
- Create a new "Cargo" Run/Debug configuration:
  - Use the workspace/package target (binary) and choose "Debug".
  - RustRover will build and launch the binary with a debugger attached.

## Notes
- If your package binary name differs from the repo name, change the program path to target/debug/<binary-name>.
- Use cargo build / cargo run for quick iteration; use CodeLLDB + rust-analyzer or RustRover for debugging.
- OS that been working on are `Linux(Arch Linux, Cachy OS)` and `Windows`.

That's it — build with cargo, run with cargo run, and use the IDE tools above to debug.
