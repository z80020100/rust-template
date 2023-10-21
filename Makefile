PROJECT_NAME := rust-template
TARGET := arm-unknown-linux-gnueabi

target/debug/${PROJECT_NAME}: src/main.rs
	cargo build

target/release/${PROJECT_NAME}: src/main.rs
	cargo build $(RUSTPROFILE)

release: target/release/${PROJECT_NAME}
release: RUSTPROFILE := --release

/target/${TARGET}/release/${PROJECT_NAME}: src/main.rs
	cross build --target ${TARGET} --release

cross: /target/${TARGET}/release/${PROJECT_NAME}

.PHONY: clean
clean:
	cargo clean

.PHONY: setup
setup:
	@echo "Setup Rust and cross compilation environment"
	@scripts/envsetup.sh
