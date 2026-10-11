# The actual machinery

Rust's standard library backs command execution, local worker channels, TCP, the bounded
HTTP exporter, the embedded novel, and score rendering. The Unix TUI provides the animated
stage and a bounded asynchronous child job. Local prompt/music helpers are subprocesses.

| Boundary | Effect |
| --- | --- |
| Normal commands | Real `/bin/sh` processes with current user permissions |
| TUI playback | Real OS audio-player child; cancellation uses job process group |
| Novel and psyche | Session-only story state and fictional character interpretation |
| Exporter | Independent HTTP process reporting its own uptime and requests |
| Synthetic graphs and props | Local simulation with no measurement claim |
| Roblox annex | Separate Luau project built with optional Roblox tools |
| Container/Helm adapters | Package and run the exporter; no interactive desktop attached |

The zero-crate claim concerns external Cargo dependencies. It does not mean no OS tools,
no embedded assets, zero binary size, or no optional tooling downloads. The original WAV
adds roughly 9 MB to the source archive and release executable.
