# The unnecessarily large operator toolbox

This is a Rust application with an optional Roblox annex. Configuration files are
provided for tools you may choose to install; they are not a claim that every tool
has run or that every IDE recognizes every directory. Orientation-only editor folders
say so in their README. Shared settings avoid credentials and machine-specific paths.

| Family | Entries |
| --- | --- |
| Rust | Cargo, rustup, rustfmt, Clippy, cargo-deny, Taplo, mise |
| Task runners | Make, just, Task, Earthly |
| Containers | Docker, Compose, Kubernetes, Kustomize, Helm, Prometheus |
| Editors | VS Code, RustRover/IntelliJ `.idea`, Zed, Helix, Neovim, Vim, Emacs, Kate, Sublime |
| Editor orientation markers | Fleet, Lapce, Nova |
| Coding agents | Codex, Junie, Claude, Cursor, Windsurf, Cline, Continue, Roo, Kilo, Amazon Q, Trae, Augment, Gemini, Copilot, Aider |
| Prose/config checks | Prettier, markdownlint, yamllint, ShellCheck, typos, codespell, Vale |
| Optional hooks | Lefthook or pre-commit; neither is installed automatically |
| Dependency automation | Renovate and Dependabot; choose one service to avoid duplicate updates |
| Roblox | Rokit, legacy Aftman, Rojo, Wally, Selene, StyLua, Lune, luau-lsp, darklua |

Start with `mise run check`, `mise run test`, and `mise run smoke`. Use
`python3 scripts/validate-configs.py` for structural checks. Optional PyYAML enables YAML
parsing; Helm and provider-specific semantics require native validators.

Codex reads trusted project defaults from `.codex/config.toml`; its instructions remain
in root AGENTS.md. Junie uses `.junie/AGENTS.md`, with a legacy guidelines entry point.
Claude uses root CLAUDE.md and shared `.claude/settings.json`. Local session state remains
ignored. No agent config installs a plugin or grants additional system access.

References: [Codex project config](https://learn.chatgpt.com/docs/config-file/config-basic),
[Junie guidelines](https://junie.jetbrains.com/docs/guidelines-and-memory.html),
[Claude settings](https://code.claude.com/docs/en/settings).
