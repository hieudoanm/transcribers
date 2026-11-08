.PHONY: build test clippy fmt run clean

build:
	cargo build --release
	@echo ""
	@echo "Binary sizes:"
	@ls -lh target/release/transcribers | awk '{printf "  %s  %s\n", $$5, $$9}'

test:
	cargo test --workspace

clippy:
	cargo clippy --workspace --all-targets

fmt:
	cargo fmt --all

install:
	./scripts/install-local.sh

run:
	cargo run --release -- $(ARGS)

clean:
	cargo clean
