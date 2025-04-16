# justfile
set dotenv-load := true

default:
    just build

build:
    cargo build --workspace

release:
    cargo build --release --workspace

test:
    cargo test --workspace

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