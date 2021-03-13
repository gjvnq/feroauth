.PHONY: help prepare run

help:
	@echo "Available targets:"
	@echo "- help"
	@echo "- prepare"
	@echo "- run"

prepare:
	rustup override set nightly
	cargo install cargo-watch
	cargo fetch

run-dev:
	cargo watch -x run -w src -w src/model -w templates -w Makefile