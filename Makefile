PROJECT_NAME := rust-template
TARGET := arm-unknown-linux-gnueabi

.PHONY: build
build:
	cargo build

.PHONY: release
release:
	cargo build --release

.PHONY: cross
cross:
	cross build --target ${TARGET} --release

.PHONY: tauri-dev
tauri-dev:
	cargo tauri dev -f tauri -- --bin ${PROJECT_NAME}-tauri

.PHONY: tauri-build
tauri-build:
	cargo tauri build

.PHONY: clean
clean:
	cargo clean
	rm -rf frontend/dist

.PHONY: setup
setup:
	@echo "Enter the following command to setup Rust and cross compilation environment"
	@echo "source scripts/envsetup.sh"
