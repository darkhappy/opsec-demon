# Roblox / the Krillion annex

The annex now creates an anchored `OpSecULAR97Mainframe` terminal in Workspace,
with `MainframeId`, `OperatingSystem`, `SerialBus`, and `City` attributes. Approach
and activate its proximity prompt to identify the shared mainframe and print a local
SIM connection to Larp City, the serial loopback, Lost Signals, and Krillion's invented
support stats into Studio's console. A surface display and player briefing show the
operator interface, Recycle Bin and WSL 中文 labels, and fire emojis. These Roblox
labels are props. The client now opens a complete Luau Nightfall terminal with an
interactive in-memory recycle bin and Chinese console under `opsecular`. There is no
external mainframe service, serial-device connection,
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
Nightfall terminal; they do not run shell commands or collect telemetry. Wally has an
empty dependency table. Lune checks the identity and command state machines outside
Studio. Selene uses Roblox
standard definitions; StyLua formats the annex; luau-lsp and darklua are optional pinned
extras. Use `rojo sourcemap default.project.json` for a language-server map if needed.

`aftman.toml` provides a legacy alternative. Prefer one manager per environment; the
manual GitHub job uses Rokit. Some validators fetch definitions or tools during setup.
No Roblox API key or universe ID is included.

## Nightfall command center

`roblox/src/client/Terminal.luau` builds the responsive interface, rotating demon seal,
alternate scene illustrations, live decorative scope, mission card, reactor meters,
and cartoon confetti. Everything remains under the existing Rojo mapping. Enter a
command and press Enter or EXEC. COMMANDS opens a searchable deck; selecting a command
fills the input for editing. Tab or the clickable hint cycles completions; Up/Down
recalls session history.
Ctrl+L clears output, and backtick hides/shows the terminal. `exit` minimizes it; the
launcher and mainframe proximity prompt reopen it.

Shared `Session.luau`, `Catalog.luau`, and `Opsecular.luau` implement local state:
operator lore, three palettes, four scenes, five mascots, branching Lost Signals,
Golden Goose incubation/buffs, mock bucket objects, dairy mesh, 512 GB virtual bank,
fictional SSD rolls, rhythm controls, fixed paper coin ledger, storage arithmetic,
country focus timers, silent band, phantom pedal, seasonal yoggies, and OS props.
`Story.luau` copies all twelve pages from `assets/story/lost-signals.md`; update both
when editing that story. The Christmas berry bonus uses UTC in this port.

`graph room` shows the idealized Sabine decay model, `graph headphones` shows invented
unmeasured EQ, and `graph metrics` shows actual local command outcomes. Signal bars
are decorative. `sdvx play` enables touch buttons and D/F/J/K/Q/E phantom hits when
the command input is unfocused; arrows adjust knobs. Every hit is a pretend CRITICAL.

Desktop shell execution, environment variables, filesystem navigation, TCP uplinks,
Discord IPC, USB discovery, desktop media, audio rendering, and the native advanced
combat calculator are explicitly unavailable. Unknown commands never fall through to
a shell. No external bridge is provisioned. Each player's state stays on their client,
resets on rejoin, and has no gameplay authority or real financial/hardware effects.

Validation: `lune run roblox/tools/session-tests.luau`, `stylua --check roblox`,
`selene roblox/src`, and `rojo build default.project.json --output roblox/build/nightfall.rbxlx`.
The shell check script also runs the session tests. The desktop PTY smoke suite imports
Unix-only `fcntl` and requires Linux/macOS; Studio tests cover the Roblox interface.

Primary references: [Rokit](https://github.com/rojo-rbx/rokit),
[Rojo project format](https://rojo.space/docs/v7/project-format/),
[setup-rokit](https://github.com/paradoxum-games/setup-rokit).
