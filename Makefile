PROJECT_NAME := rust-template

target/debug/${PROJECT_NAME}: src/main.rs
	cargo build

target/release/${PROJECT_NAME}: src/main.rs
	cargo build $(RUSTPROFILE)

release: target/release/rust-template
release: RUSTPROFILE := --release

.PHONY: clean
clean:
	cargo clean

.PHONY: setup
setup:
	@echo "Setup Rust and cross compilation environment"
	@scripts/envsetup.sh
