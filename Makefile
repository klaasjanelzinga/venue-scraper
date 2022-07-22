.PHONY: tests

tests:
	RUST_BACKTRACE=1 cargo test

fix:
	cargo fmt --all
	cargo check
	cargo check --tests
