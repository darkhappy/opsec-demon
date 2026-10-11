# Native Windows Vulkan concert deck

Double-click `OPSEC-DEMON.exe` from `target/dist`. The Windows executable opens
the graphical command deck by default. It embeds the original Nightfall score
and draws its artwork directly at the client resolution; no browser, Python,
WSL, font installation, or files beside the executable are needed for the deck.
Windows 10/11 x64 with a Vulkan-capable graphics driver is the supported platform.
Update the GPU manufacturer's driver if Vulkan initialization fails.

The interface uses egui/wgpu with **Vulkan exclusively**. Its procedural shader
draws neon rings, perspective grids, laser beams and celebration waves; vector
illustrations animate the Beatles tribute and drummer fragmentation. Effects
are SIM illustrations, not hardware events. Seven pages separate the command
deck, concert, audio lab, rhythm arcade, story, props and integrations.

F11 toggles fullscreen; Escape cancels a child command and its process tree.
Ctrl-G stomps the pedal and Ctrl-X explodes the drummer when a text field is not
focused. The Arcade uses D/F/J/K for BT-A/B/C/D and A/L for FX-L/R. Notes are
decorative; the existing phantom controller awards its original pretend score.
Enter executes commands; Tab completes and Up/Down navigate session history.
Mouse-wheel scrolling reviews long pages and transcripts. Interactive terminal
programs use `--shell`; the Unix TUI remains available on Unix.

VSync is the default and the app redraws continuously at the available refresh
rate. Set `OPSEC_DEMON_UNCAPPED=1` before launch for Vulkan's AutoNoVsync mode.
The sidebar reports observed update cadence and CPU time spent constructing
the UI, not GPU timing. Actual performance depends on resolution, driver and
hardware; the smoke benchmark reports offscreen render throughput separately.

Existing story and simulation commands are available in the command panel:
`novel start`, `novel next`, `yoggies status`, `opsecular boot`, `mission next`,
`rips list`, `theme venom`, and `soundtrack play`. The buttons invoke existing
commands and shared models. The drummer's original explosion/reassembly timer,
pedal state, headphone selection, SSD disaster, story choices, country deck,
Golden Egg, yoggies, paper coins and other props are preserved. Command Deck
retains the full builtin command surface, including advanced TTK and mainframe
commands. Output is bounded to 1,500 visible transcript lines and 4,000 characters
per line; queue drops are counted in the status bar.

## Audio graphs

Audio Lab displays the existing analytic low-pass **frequency response, phase,
group delay and excess group delay**, with cutoff/Q/transport-delay controls.
It also plots the selected invented headphone EQ and Sabine room decay with
editable dimensions/absorption. These models are explicitly unmeasured.
Two additional plots analyze a mono window of the embedded original score:
PCM amplitude and a Hann-windowed spectrum. Their timeline is estimated from
app uptime; they do not capture Apple Music, a microphone, or the output device.
Command Deck adds a graph of actual session frame cadence and session counters.

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
- **Audio:** the original Nightfall score loops automatically through WinMM from
  embedded PCM memory. The sidebar toggles playback; closing the app stops it.
  This is an original composition, not a Beatles recording. `soundtrack play`
  stops that loop, extracts the embedded score to a temporary file
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

The Windows tasks use Rust 1.93.1's GNU target and bundled LLVM linker, with small
import-library/linker adapters under `scripts/`; Visual Studio Build Tools and
an external assembler are not required. Full `mise run smoke` requires a Vulkan
adapter. It checks real command output and shared model effects, then renders
all seven pages offscreen at 3840 x 2160. It writes `target/desktop-4k.png`,
`target/desktop-page-1-4k.png` through `target/desktop-page-6-4k.png`, and
`target/desktop-audio-source-4k.png`, plus `target/vulkan-smoke.json`. It then
opens the native Vulkan window for 120 updates and closes it automatically.
The report includes adapter identity and a cached-UI
render-throughput experiment without display/VSync or PNG encoding; it is not
an interactive FPS claim. Audio playback is muted during the smoke.

Hosted Windows CI runs native command smoke with explicit `-SkipVulkan`, since
it does not provide the required Vulkan adapter; local full smoke validates GPU
rendering. Packaging writes the standalone executable and a SHA-256 checksum
to `target/dist`; build output remains ignored by Git.

To retain a terminal session:

```powershell
.\target\dist\OPSEC-DEMON.exe --shell
.\target\dist\OPSEC-DEMON.exe --plain -c 'echo hello | findstr hello'
```

For a machine with the pinned Rust target already installed, the equivalent
build is `powershell.exe -NoProfile -ExecutionPolicy Bypass -File
scripts/rust-windows.ps1 build --release --locked`.
