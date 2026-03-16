# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build                # Debug build
cargo build --release      # Release build
cargo run                  # Run (reads configs/main.toml relative to working directory)
cargo fmt --all -- --check # Check formatting
cargo clippy --all-targets # Lint (all targets including tests, benches, examples)
cargo test                 # Run all tests
cargo test <test_name>     # Run a single test

# Cross-compilation (requires `cross` tool)
make cross                              # Default: arm-unknown-linux-gnueabi
make cross TARGET=aarch64-unknown-linux-gnu  # Specify target
make setup                              # Show setup instructions for cross env
```

`.cargo/config.toml` sets `rustflags = ["-Dwarnings"]` — all warnings are treated as errors everywhere (local dev, CI, pre-commit hook).

## Architecture

This is a Rust application template built on **Tokio** async runtime with structured logging via **tracing**.

### Startup Flow (`src/main.rs`)

1. Initialize logger with default DEBUG level (console + file)
2. Load config from `configs/main.toml`
3. Reconfigure logger levels from loaded config
4. Enter Tokio multi-threaded runtime → spawn threads → wait for completion
5. Map `Result` to `ExitCode` (success → 0, error → non-zero via `ErrorCode::as_u8()`)

### Module Overview

- **`configs`** — Loads `MainConfig` from TOML via `config` + `serde`. Config path is relative to the working directory.
- **`logger`** — Dual-output logger (stderr console with ANSI colors + daily rotating file in `log/`). Both output levels are independently reloadable at runtime via `reconfig()`. The `WorkerGuard` must be held alive in `main()`.
- **`error`** — `ErrorCode` enum (errors only, no `Success` variant). `as_u8()` maps each variant to a non-zero exit code. `main()` returns `ExitCode` by mapping `Ok(()) → SUCCESS`, `Err(e) → e.as_u8()`. Thread functions return `Result<(), ErrorCode>`.
- **`threads`** — Thread management with `JoinSet`. Three demo threads: producer (sends i32 via MPSC), consumer (receives from MPSC), signal_handler (OS signals). All threads listen on a broadcast channel for `ThreadCommand::Stop`.
- **`constant`** — Shared constants (e.g., broadcast channel capacity).

### Threading Pattern

- **Command channel**: `broadcast::channel<ThreadCommand>` — all threads subscribe; used to send `Stop`
- **Data channel**: `mpsc::unbounded_channel<i32>` — producer→consumer
- Each thread uses `tokio::select!` to concurrently await data and stop commands
- If any thread returns an `Err(ErrorCode)`, remaining threads are stopped

### Design Conventions

- **Intentional duplication in thread modules**: Each thread submodule (producer, consumer, signal_handler) has its own `cmd_handler` function. These are intentionally kept separate — do NOT extract them into a shared helper. This is a template project; users will fork and customize each thread independently.

### Platform-Specific Code

Uses `cfg_if!` in `signal_handler.rs`: Unix signals (SIGTERM/SIGINT/SIGHUP) vs Windows control events (CTRL+C/CTRL+BREAK/CTRL+CLOSE).

### Pre-commit Hook

**cargo-husky** with `user-hooks` feature reads custom hooks from `.cargo-husky/hooks/` in the repo and installs them into `.git/hooks/` on first `cargo test`. The pre-commit hook runs fmt check, clippy, and tests.

### Cross-Compilation

- `Cross.toml` defines custom Docker images per target (see `cross/docker/`)
- Default cross target: `arm-unknown-linux-gnueabi` (Raspberry Pi Zero W)
- CI has commented-out steps for `aarch64-unknown-linux-gnu`
