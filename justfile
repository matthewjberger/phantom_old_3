set windows-shell := ["powershell.exe"]

export RUST_BACKTRACE := "1"
export RUST_LOG := "info"

editor:
  cargo run -r --bin editor

check:
  cargo check --workspace --tests
  cargo fmt --check

format:
  cargo fmt --all

lint:
  cargo clippy

test:
  cargo test --workspace

@versions:
  rustc --version
  cargo fmt -- --version
  cargo clippy -- --version

viewer:
  cargo run -r --bin viewer
