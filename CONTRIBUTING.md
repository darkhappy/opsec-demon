# Enter the forge

Read [AGENTS.md](AGENTS.md), then build with `mise run build`. Rustfmt and Clippy
run through `mise run check`; unit tests use `mise run test`; the PTY suite uses
`mise run smoke`. The opt-in socket test runs with `mise exec -- cargo test -- --ignored`.

Commit each atomic change separately. Describe what changed, why, and what you actually
verified. Keep simulated commands visibly fictional. Runtime changes need relevant
checks; prose and artwork changes need link and asset checks. Optional Prettier settings
cover documentation; Rust remains under rustfmt. No Node project is required.

For terminal bugs, include OS, terminal, rows/columns, font mode, launch flags, and a
minimal reproduction with secrets removed. New decorative personas should remain local
and respect small layouts. Never turn a fictional badge into a promise of security.
