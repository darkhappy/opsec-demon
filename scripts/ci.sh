#!/bin/sh
# DEUS EX MACHINA: actual checks behind fictional clearance.
set -eu
cd "$(dirname "$0")/.."
command -v python3 >/dev/null
rustup component add --toolchain 1.93.1 rustfmt clippy
rustup run 1.93.1 cargo fmt --check
rustup run 1.93.1 cargo clippy --all-targets -- -D warnings
rustup run 1.93.1 cargo test --locked
if [ "${OPSEC_CI_SOCKETS:-0}" = 1 ]; then
    rustup run 1.93.1 cargo test --locked -- --ignored
fi
rustup run 1.93.1 cargo build --release --locked
if [ "${OPSEC_CI_SOCKETS:-0}" = 1 ]; then
    python3 tests/metrics_smoke.py
fi
python3 scripts/validate-configs.py
python3 tests/tui_smoke.py --boot
