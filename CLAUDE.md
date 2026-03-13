# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build                # Debug build
cargo build --release      # Release build
cargo run                  # Run (reads configs/main.toml relative to working directory)
cargo fmt --all -- --check # Check formatting
cargo clippy               # Lint
cargo test                 # Run all tests
cargo test <test_name>     # Run a single test

# Cross-compilation (requires `cross` tool)
make cross                              # Default: arm-unknown-linux-gnueabi
make cross TARGET=aarch64-unknown-linux-gnu  # Specify target
make setup                              # Show setup instructions for cross env
```

CI sets `RUSTFLAGS: -Dwarnings` — all warnings are treated as errors in CI.

## Architecture

This is a Rust application template built on **Tokio** async runtime with structured logging via **tracing**.

### Startup Flow (`src/main.rs`)

1. Initialize logger with default DEBUG level (console + file)
2. Load config from `configs/main.toml`
3. Reconfigure logger levels from loaded config
4. Enter Tokio multi-threaded runtime → spawn threads → wait for completion
5. Return `ErrorCode` as process exit code

### Module Overview

- **`configs`** — Loads `MainConfig` from TOML via `config` + `serde`. Config path is relative to the working directory.
- **`logger`** — Dual-output logger (stderr console with ANSI colors + daily rotating file in `log/`). Both output levels are independently reloadable at runtime via `reconfig()`. The `WorkerGuard` must be held alive in `main()`.
- **`error`** — `ErrorCode` enum with `#[repr(u8)]` discriminants. Implements `Termination` trait so it can be returned directly from `main()`. Thread functions return `Result<(), ErrorCode>`.
- **`threads`** — Thread management with `JoinSet`. Three demo threads: producer (sends i32 via MPSC), consumer (receives from MPSC), signal_handler (OS signals). All threads listen on a broadcast channel for `ThreadCommand::Stop`.
- **`constant`** — Shared constants (e.g., broadcast channel capacity).

### Threading Pattern

- **Command channel**: `broadcast::channel<ThreadCommand>` — all threads subscribe; used to send `Stop`
- **Data channel**: `mpsc::unbounded_channel<i32>` — producer→consumer
- Each thread uses `tokio::select!` to concurrently await data and stop commands
- If any thread returns a non-Success `ErrorCode`, remaining threads are stopped

### Design Conventions

- **Intentional duplication in thread modules**: Each thread submodule (producer, consumer, signal_handler) has its own `cmd_handler` function. These are intentionally kept separate — do NOT extract them into a shared helper. This is a template project; users will fork and customize each thread independently.

### Platform-Specific Code

Uses `cfg_if!` in `signal_handler.rs`: Unix signals (SIGTERM/SIGINT/SIGHUP) vs Windows control events (CTRL+C/CTRL+BREAK/CTRL+CLOSE).

### Cross-Compilation

- `Cross.toml` defines custom Docker images per target (see `cross/docker/`)
- Default cross target: `arm-unknown-linux-gnueabi` (Raspberry Pi Zero W)
- CI has commented-out steps for `aarch64-unknown-linux-gnu`
