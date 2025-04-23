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
    cargo test --workspace --all-features --all-targets

nextest:
    cargo nextest run --all-features --all-targets --workspace --no-fail-fast --test-threads=-6

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
    cargo check --workspace --all-features --all-targets

update:
    cargo update && cargo outdated || true

cover:
    cargo llvm-cov --workspace --all-features --all-targets

# Run GitHub Actions workflows locally using act
act workflow="":
    if [ "${workflow:-}" = "" ]; then \
        act; \
    else \
        act --workflows "${workflow}"; \
    fi