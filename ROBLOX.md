# Roblox / the Krillion annex

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
