.PHONY: build check test smoke ci run configs roblox
build:
	cargo build --release --locked
check:
	cargo fmt --check
	cargo clippy --all-targets -- -D warnings
test:
	cargo test --locked
smoke: build
	python3 tests/tui_smoke.py --boot
ci:
	sh scripts/ci.sh
run:
	cargo run --release --
configs:
	python3 scripts/validate-configs.py
roblox:
	sh scripts/roblox-check.sh
