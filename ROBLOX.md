# Roblox / the Krillion annex

The annex now creates an anchored `OpSecULAR97Mainframe` terminal in Workspace,
with `MainframeId`, `OperatingSystem`, `SerialBus`, and `City` attributes. Approach
and activate its proximity prompt to identify the shared mainframe and print a local
SIM connection to Larp City, the serial loopback, Lost Signals, and Krillion's invented
support stats into Studio's console. A surface display and player briefing show the
operator interface, Recycle Bin and WSL 中文 labels, and fire emojis. These Roblox
labels are props; the interactive recycle bin and Chinese console live in the Rust
`opsecular` command. There is no external mainframe service, serial-device connection,
original Tama-soft game integration, or network bridge to the Rust process.

Install Rokit 1.2.0 from its official releases and place it on PATH. This optional annex
is independent of Cargo and has no upload/deployment step.

```sh
rokit install
sh scripts/roblox-check.sh
rojo serve default.project.json
# In Studio, connect using the Rojo plugin.
```

Rojo builds `roblox/build/nightfall.rbxlx`; the output stays ignored. The shared Identity
module describes the fictional operation. Server and client scripts print/show a local
Nightfall briefing; they do not run shell commands or collect telemetry. Wally has an
empty dependency table. Lune checks the shared module outside Studio. Selene uses Roblox
standard definitions; StyLua formats the annex; luau-lsp and darklua are optional pinned
extras. Use `rojo sourcemap default.project.json` for a language-server map if needed.

`aftman.toml` provides a legacy alternative. Prefer one manager per environment; the
manual GitHub job uses Rokit. Some validators fetch definitions or tools during setup.
No Roblox API key or universe ID is included.

Primary references: [Rokit](https://github.com/rojo-rbx/rokit),
[Rojo project format](https://rojo.space/docs/v7/project-format/),
[setup-rokit](https://github.com/paradoxum-games/setup-rokit).
