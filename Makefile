.DEFAULT_GOAL := build
.PHONY: check clean clippy doc fmt test

build:
	cargo build --all-features --all-targets --benches --bins --examples --tests --workspace

check:
	cargo check --all-features --all-targets --benches --bins --examples --tests --workspace

clean:
	cargo clean

clippy:
	cargo clippy --all-features --all-targets --benches --bins --examples --tests --workspace -- -D warnings

doc:
	cargo +nightly doc --all-features --open

fmt:
	cargo +nightly fmt --all

test:
	cargo test --all-features --all-targets --benches --bins --examples --tests --workspace
