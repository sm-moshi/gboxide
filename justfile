# justfile
set dotenv-load := true

default:
    just build

build:
    cargo build --workspace

release:
    cargo build --release --workspace

# Run all tests (unit, integration, Mooneye)
test:
    cargo test --workspace

nextest:
    cargo nextest run --workspace --no-fail-fast --test-threads=-4

fmt:
    cargo fmt --all

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

clean:
    cargo clean

watch:
    cargo watch -x run

bench:
    cargo bench --workspace

lint: fmt clippy

run:
    cargo run -p cli

check:
    cargo check --workspace --all-targets

update:
    cargo update && cargo outdated || true

cover:
    cargo llvm-cov