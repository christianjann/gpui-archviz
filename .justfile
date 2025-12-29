default:
  just --list

set shell := ['bash', '-cu']

fmt:
    cargo fmt --all

clippy:
    cargo clippy --all-targets -- -D warnings

fix:
    cargo clippy --all-targets --fix --allow-dirty

build:
    cargo build

build-release:
    cargo build --release

run:
    cargo run --

test:
    cargo test

clean:
    cargo clean

doc:
    cargo doc --open

install:
    cargo install --path .

watch:
    cargo watch -x 'run'

layout-test:
    cargo test -p layout -- --nocapture