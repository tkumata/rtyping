fmt:
	cargo fmt

fmt-check:
	cargo fmt --check

lint:
	cargo clippy -- -D warnings

test:
	cargo test --all-features

check: fmt-check lint test

build:
	cargo build --all-features
