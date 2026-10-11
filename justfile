# Optional just recipes; no tools are installed implicitly.
default:
    @just --list
build:
    cargo build --release --locked
check:
    cargo fmt --check
    cargo clippy --all-targets -- -D warnings
test:
    cargo test --locked
ci:
    sh scripts/ci.sh
roblox:
    sh scripts/roblox-check.sh
