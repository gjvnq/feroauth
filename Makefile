.PHONY: help prepare run

RUSTFLAGS_DEV="-C link-arg=-fuse-ld=lld"

help:
	@echo "Available targets:"
	@echo "- help"
	@echo "- prepare"
	@echo "- run"

prepare:
	rustup override set stable
	cargo install cargo-watch
	cargo fetch

run-dev:
	RUSTFLAGS=$(RUSTFLAGS_DEV) cargo watch -x run -w src -w src/model -w templates -w Makefile -w .env

build-dev:
	RUSTFLAGS=$(RUSTFLAGS_DEV) cargo build

test:
	RUSTFLAGS=$(RUSTFLAGS_DEV) cargo test