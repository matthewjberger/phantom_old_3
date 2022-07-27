# Disable this if not running on windows
set shell := ["powershell.exe", "-c"]

export RUST_BACKTRACE := "1"

editor:
  cargo run -r --bin editor

check:
  cargo check --workspace

format:
  cargo fmt --all

test:
    cargo test

versions:
    rustc --version
    cargo fmt -- --version
    cargo clippy -- --version

viewer:
  cargo run -r --bin viewer

