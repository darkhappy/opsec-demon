#!/bin/sh
set -eu
printf '[1/3] LOADING RUST FORGE / checking compiler\n'
cargo --version
printf '[2/3] ASSEMBLING NIGHTFALL / compiling and linking Rust binary\n'
cargo build --release
printf '[3/3] CONSTRUCT READY / binary bytes: '
wc -c < target/release/opsec-demon
printf 'Launch ./target/release/opsec-demon --nerd-font\n'
