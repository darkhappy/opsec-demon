# Native Windows command deck

Double-click `OPSEC-DEMON.exe` from `target/dist`. The Windows executable opens
the graphical command deck by default. It embeds the original Nightfall score
and draws its artwork directly at the client resolution; no browser, Python,
WSL, font installation, or files beside the executable are needed for the deck.
Windows 10/11 x64 is the supported native platform.

The neon demon, radar, and signal waveform are procedural illustrations labelled
SIM. The artwork scales to 3840 x 2160 and higher client resolutions. F11 toggles
fullscreen on the current monitor. Pause stops the decorative animation. This
is a 2D command deck; it does not claim photorealistic 3D graphics or a measured
frame rate. Mouse-wheel scrolling reviews the command transcript. The input
supports native Windows Unicode entry and paste; press Enter or click EXECUTE.
STOP or Escape cancels the running local command and its process tree.

Existing story and simulation commands are available in the command panel:
`novel start`, `novel next`, `yoggies status`, `opsecular boot`, `mission next`,
`rips list`, `theme venom`, and `soundtrack play`. The buttons invoke existing
commands. Long transcript lines are visually truncated; wider windows reveal
more text, and `--shell` provides a terminal for full-width output and interactive
programs. The legacy `--tui` remains a Unix interface.

## Real local integrations

- **Commands:** Windows runs real `cmd.exe /D /S /C` commands, including pipes,
  redirects, quoted paths, and exit statuses. Unix continues to run `/bin/sh`.
  Use `cd "C:\Users\operator\space folder"` for native paths. `export` and `unset`
  are builtins that control child environments. `shell` requires terminal mode.
- **Apple Music:** the embedded PowerShell adapter reads Windows media sessions.
  Keep Apple Music open. The deck shows its title, artist, and playing/paused
  state without the WSL MPRIS bridge or API credentials. `music demo` explicitly
  selects fictional metadata; `music live` restores the local source. A closed
  player or unavailable session is reported honestly. Local playback telemetry
  does not implement Apple catalog, library, or remote playback APIs.
- **Discord:** local named-pipe IPC supports the same validated Rich Presence
  profiles, animated assets, acknowledgement, and retry behavior as Unix. Use
  `discord on APPLICATION_ID`, `discord status`, `discord preview`, and
  `discord off`. A public application ID and the running Discord desktop client
  are required; no bot token or client secret is used. Setting
  `DISCORD_APPLICATION_ID` before launch enables presence automatically.
  All theatrical presence claims remain labelled SIM.
- **Audio:** `soundtrack play` extracts the embedded score to a temporary file
  and plays through the Windows default output device. Explicit WAV paths work
  as well. The player removes the extracted file on normal completion. Forced
  cancellation may leave the temporary WAV until the OS cleans temporary files.
  DAC/serial commands do not claim Windows hardware discovery.

Environment variables are inherited from the process that launches the app;
`.env` files are not read automatically. Keep credentials out of Git.

## Rebuild and validate

With mise installed, from PowerShell in the repository:

```powershell
mise trust
mise install
mise run windows:setup
mise run check
mise run test
mise run smoke
mise run windows:package
```

The Windows tasks use Rust 1.93.1's GNU toolchain and bundled linker, so Visual
Studio Build Tools are not required. Smoke opens a hidden native window,
checks real command output and SIM consoles, and saves a 3840 x 2160 render to
`target/desktop-4k.bmp`. Packaging writes the standalone executable and a SHA-256
checksum to `target/dist`; build output remains ignored by Git.

To retain a terminal session:

```powershell
.\target\dist\OPSEC-DEMON.exe --shell
.\target\dist\OPSEC-DEMON.exe --plain -c 'echo hello | findstr hello'
```

For a machine with Rust already installed, the equivalent build is
`cargo +1.93.1-x86_64-pc-windows-gnu build --release --locked`.
