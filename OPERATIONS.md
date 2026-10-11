# Local runbook / no remote operation implied

Build with `mise run build`; validate with `sh scripts/ci.sh`. Set OPSEC_CI_SOCKETS=1 to
include optional loopback checks on a permissive runner. The Rust app targets Unix
terminals on Linux and macOS. Run the Roblox annex only after installing its tools.

Use Ctrl-D for normal TUI extraction and Ctrl-C for the active child. After an
uncatchable process kill, `stty sane` can restore input. `--instant` skips the cinematic.
Render soundtrack files to a new path; existing files are not overwritten.

For exporter incidents, check `/healthz` and `/metrics`, process logs, and the container
port mapping. Kubernetes readiness/liveness probes use the same endpoint contract.
Load or publish the image into the selected cluster's accessible image store before
installing the manifests or chart. No remote has been configured by this work.

Each source/config/prose change gets a new, non-empty commit. Follow-up edits receive new
commits too. Timestamps are sampled from the clock after it advances past the previous
commit; the earlier history is not rewritten. Review `git log --format='%h %aI %s'`.
