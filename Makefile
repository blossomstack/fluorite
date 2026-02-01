.PHONY: build test clean fmt lint check all release

all: fmt-check lint test

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

clean:
	cargo clean

fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

lint:
	cargo clippy --all-targets --all-features -- -D warnings

check:
	cargo check
