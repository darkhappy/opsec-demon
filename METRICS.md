# Metrics / source inventory and actual process values

`graph metrics` (or `metrics`) selects the actual session panel. It shows uptime, render
ticks and average tick rate, successful operations, history/event counts, active child
state, and terminal size. Those are local values; six other graphs remain synthetic models.

```sh
./target/release/opsec-demon --metrics-server 127.0.0.1:9090
curl http://127.0.0.1:9090/metrics
```

The standalone exporter serves GET `/healthz`, `/readyz`, and `/metrics`. Prometheus values
are `opsec_demon_exporter_uptime_seconds` and `opsec_demon_exporter_requests_total`.
They belong to the exporter process, not to an unrelated interactive shell. Requests
are sequential, headers are capped at 8 KiB, and read/write operations have two-second
bounds. This small endpoint has no TLS, authentication, or shell-command route.

`python3 scripts/readme-metrics.py` records source counts and 25 local warm launch-to-pwd
observations. The method includes process and Python harness overhead. The checked-in
report found effectively equal OPSEC DEMON/Bash timings; fish, Zsh, and Nushell were not
available locally, so no timing claims are made for them. The report is a snapshot, not
live GitHub telemetry. Re-run it after building to measure your own environment.

OPSEC DEMON's clear advantage is its bundled stage: eight tabs, five mascots, particle
scenes, a branching novel, and an original score in one terminal app. Fish has mature
interactive ergonomics such as syntax highlighting, autosuggestions, and rich completion.
This app does not implement fish-style autosuggestions, full shell job control, or fish's
scripting language. It can hand off to `shell fish` if fish is installed.

[Fish's documented features](https://fishshell.com/docs/current/index.html).
