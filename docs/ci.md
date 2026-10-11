# Continuous integration

Every push to every branch and every pull request runs five GitHub Actions workflows,
with independent results so a failure identifies the affected area. Manual dispatch
is also available. New commits cancel superseded runs on the same branch. Jobs have
explicit time limits and read-only repository permissions; container images are
built for verification and are not published.

| Workflow | Checks |
| --- | --- |
| Rust quality | Formatting/Clippy, unit tests, and documentation on Linux and macOS (six checks) |
| Runtime integration | Full PTY/boot/command/terminal restoration smoke and socket/HTTP exporter contracts on Linux and macOS (four checks) |
| Repository validation | Configuration parsing and local document references, Python syntax, POSIX shell syntax, and GitHub Actions validation (four checks) |
| Container verification | Production Docker build, unprivileged user, health/readiness, and Prometheus endpoint (one check) |
| Roblox annex | StyLua, Selene, Lune session tests, and Rojo packaging (one check) |

Rust jobs use the repository's mise pins. The Roblox job uses rokit.toml.
Configuration checks install PyYAML so YAML parsing is exercised rather than skipped.
Actionlint validates workflow expressions and job structure. Shell and Python syntax
checks do not claim to exercise external service integrations or provider schemas.

The GitHub main branch currently contains the Unix terminal application. Native Windows
work in local branches needs its own Windows runner checks when that implementation is
published. Linux/macOS PTY tests do not establish Windows desktop behavior. Roblox checks
verify scripts and packaging; interactive Studio and multiplayer behavior still need
Studio testing.

For local Rust validation run `mise run check`, `mise run test`, and `mise run smoke`.
To include the loopback socket contract run `cargo test --locked -- --ignored`, then
`python3 tests/metrics_smoke.py` after building the release executable. Run
`python3 scripts/validate-configs.py` for configuration and document reference checks.

These workflows report checks; they do not enforce branch protection. Repository
administrators can select required status checks in GitHub branch rules once the checks
have run. Dependency update pull requests run the same suites, including the Docker
build for base-image updates and actionlint for GitHub Actions updates.
