.PHONY: check
check:
	cargo check --all-features --all-targets --benches --bins --examples --tests --workspace

.PHONY: clean
clean:
	cargo clean

.PHONY: clippy
clippy:
	cargo clippy --all-features --all-targets --benches --bins --examples --tests --workspace -- -D warnings

.PHONY: fmt
fmt:
	cargo +nightly fmt --all

.PHONY: test
test:
	cargo test --all-features --all-targets --benches --bins --examples --tests --workspace
