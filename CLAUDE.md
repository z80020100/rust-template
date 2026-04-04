# rust-template

A Rust application template with two modes: **CLI** and **Tauri GUI**. Both share a common library (`src/lib.rs`) for logging and configuration. Fork and keep the mode you need.

> This file also provides guidance to [Claude Code](https://claude.ai/code) when working with code in this repository.

## Prerequisites

- Rust 1.85+ (edition 2024)
- Node.js LTS and npm (for Tauri GUI frontend)

### Tauri System Dependencies

- **Linux (Ubuntu/Debian)**: `sudo apt install libwebkit2gtk-4.1-dev build-essential libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Windows**: Microsoft Edge WebView2 (pre-installed on Windows 10/11)

## Quick Start

### CLI

```bash
cargo run
```

Runs a producer-consumer demo with signal handling. Reads config from `configs/main.toml`. Logs to stderr (console) and `log/` (rotating files).

### Tauri GUI

```bash
make tauri-dev     # Development (with hot reload)
make tauri-build   # Production build
```

Opens a GUI window with the same producer-consumer demo. In debug builds the DevTools button and backend log forwarding to the browser console are available.

## Fork Simplification

### Keep CLI Only

1. Delete `src/bin/`, `frontend/` and `tauri.conf.json`
2. In `Cargo.toml`: remove the `[features]` section and `[[bin]]` section and optional dependencies (`tauri`, `serde_json`, `tauri-build`)
3. In `build.rs`: remove the `#[cfg(feature = "tauri")]` block
4. In `.cargo-husky/hooks/pre-commit`: remove the frontend check steps
5. Update CI to remove the tauri variant

### Keep Tauri GUI Only

1. Move `src/bin/tauri/main.rs` logic into `src/main.rs`
2. Delete `src/bin/`, `src/threads/` and `src/constant.rs`
3. In `Cargo.toml`: remove `[features]` and `[[bin]]` sections. Change `tauri`, `serde_json` and `tauri-build` from optional to required
4. Update CI to remove the default variant

## Build & Test Commands

```bash
cargo build                # Debug build
cargo build --release      # Release build
cargo run                  # Run CLI
cargo fmt --all -- --check # Check formatting
cargo clippy --all-targets # Lint CLI
cargo test                 # Run all tests
cargo test <test_name>     # Run a single test

# Tauri GUI
cargo clippy --all-targets --features tauri  # Lint with Tauri
cargo test --features tauri                  # Test with Tauri
make tauri-dev                               # Dev mode (hot reload)
make tauri-build                             # Production build

# Frontend
npm run --prefix frontend format:check  # Prettier
npm run --prefix frontend lint          # ESLint + @html-eslint
npm run --prefix frontend lint:css      # Stylelint

# Cross-compilation (requires `cross` tool)
make cross                              # Default: arm-unknown-linux-gnueabi
make cross TARGET=aarch64-unknown-linux-gnu  # Specify target
make setup                              # Show setup instructions for cross env
```

`.cargo/config.toml` sets `rustflags = ["-Dwarnings"]` — all warnings are treated as errors everywhere (local dev, CI, pre-commit hook).

## Architecture

This is a Rust application template built on **Tokio** async runtime with structured logging via **tracing**. The `tauri` Cargo feature gates all GUI dependencies. Without it only the CLI binary is built.

### CLI Startup Flow (`src/main.rs`)

1. Initialize logger with default TRACE level (console + file)
2. Load config from `configs/main.toml`
3. Reconfigure logger levels from loaded config
4. Enter Tokio multi-threaded runtime → spawn threads → wait for completion
5. Map `Result` to `ExitCode` (success → 0, error → non-zero via `ErrorCode::as_u8()`)

### Tauri GUI Startup Flow (`src/bin/tauri/main.rs`)

1. Initialize logger with default TRACE level (console + file + event layer in debug)
2. Load config from `configs/main.toml` and reconfigure logger
3. Take the event log receiver from the logger
4. Build Tauri app: register IPC commands and set up the main window
5. In `.setup()`: set dynamic window title and spawn a log-forwarding task with a oneshot handshake (waits for frontend `log-ready` signal before emitting)
6. Map `Result` to `ExitCode`

### Module Overview

- **`configs`** — Loads `MainConfig` from TOML via `config` + `serde`. Config path is relative to the working directory.
- **`logger`** — Dual-output logger (stderr console with ANSI colors + daily rotating file in `log/`). Both output levels are independently reloadable at runtime via `reconfig()`. The `WorkerGuard` must be held alive in `main()`. In debug builds with the `tauri` feature an `EventLayer` sends `LogRecord` structs via `tokio::sync::mpsc` for WebView forwarding.
- **`error`** — `ErrorCode` enum (errors only, no `Success` variant). `as_u8()` maps each variant to a non-zero exit code. `main()` returns `ExitCode` by mapping `Ok(()) → SUCCESS`, `Err(e) → e.as_u8()`. Thread functions return `Result<(), ThreadErrorCode>`.
- **`threads`** — Thread management with `JoinSet`. Three demo threads: producer (sends i32 via MPSC), consumer (receives from MPSC), signal_handler (OS signals). All threads listen on a broadcast channel for `ThreadCommand::Stop`.
- **`constant`** — Shared constants (e.g., broadcast channel capacity).
- **`bin/tauri/commands`** — Tauri IPC commands (`start`, `stop`, `open_devtools`). Producer and consumer run as Tokio tasks controlled via `broadcast::channel` for stop signaling.

### CLI Threading Pattern

- **Command channel**: `broadcast::channel<ThreadCommand>` — all threads subscribe; used to send `Stop`
- **Data channel**: `mpsc::unbounded_channel<i32>` — producer→consumer
- Each thread uses `tokio::select!` to concurrently await data and stop commands
- If any thread returns an `Err(ThreadErrorCode)`, remaining threads are stopped

### Tauri IPC Pattern

- Frontend calls `invoke("start")` / `invoke("stop")` via `@tauri-apps/api/core`
- Backend emits `produce` and `consume` events via `AppHandle::emit()`
- Frontend listens with `listen()` from `@tauri-apps/api/event`
- Debug-only log forwarding: backend `EventLayer` → `mpsc` → `emit("log")` → frontend `console.*`

### Design Conventions

- **Intentional duplication in thread modules**: Each thread submodule (producer, consumer, signal_handler) has its own `cmd_handler` function. These are intentionally kept separate — do NOT extract them into a shared helper. This is a template project; users will fork and customize each thread independently.

### Platform-Specific Code

Uses `cfg_if!` in `signal_handler.rs`: Unix signals (SIGTERM/SIGINT/SIGHUP) vs Windows control events (CTRL+C/CTRL+BREAK/CTRL+CLOSE).

### Pre-commit Hook

**cargo-husky** with `user-hooks` feature reads custom hooks from `.cargo-husky/hooks/` in the repo and installs them into `.git/hooks/` on first `cargo test`. The pre-commit hook runs fmt check, clippy, tests and frontend format/lint checks.

### Cross-Compilation

- `Cross.toml` defines custom Docker images per target (see `cross/docker/`)
- Default cross target: `arm-unknown-linux-gnueabi` (Raspberry Pi Zero W)
- CI runs `aarch64-unknown-linux-gnu` cross-compilation for the default variant
