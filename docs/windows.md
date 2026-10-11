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
are SIM illustrations, not hardware events. Eight pages separate the command
deck, concert, audio lab, rhythm arcade, story, props, integrations and settings.

F11 toggles fullscreen; Escape cancels a child command and its process tree.
Ctrl-G stomps the pedal and Ctrl-X explodes the drummer when a text field is not
focused. The Arcade uses D/F/J/K for BT-A/B/C/D and A/L for FX-L/R. Notes are
decorative; the existing phantom controller awards its original pretend score.
Enter executes commands; Tab completes and Up/Down navigate session history.
Mouse-wheel scrolling reviews long pages and transcripts. Interactive terminal
programs use `--shell`; the Unix TUI remains available on Unix.

VSync is the default and the app redraws continuously at the available refresh
rate. Set `OPSEC_DEMON_UNCAPPED=1` before launch for Vulkan's AutoNoVsync mode.
The sidebar and optional FPS overlay report observed update cadence. The sidebar's
CPU measurement covers UI construction, not GPU timing. Performance depends on resolution, driver and
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

## Graphics and audio settings

Open **Graphics + Audio** for Low/Balanced/High/Ultra presets, a persistent FPS
counter, frame pacing limit, fullscreen, window sizes, interface scale, and VSync.
VSync takes effect after **Save settings** and restart; `OPSEC_DEMON_UNCAPPED=1`
overrides it. The pacing limit controls scheduled updates; input events can request
additional frames. FPS is observed GUI cadence, not a GPU timestamp measurement.

**FSR 1** uses an FP32 WGSL adaptation of AMD's EASU edge-adaptive spatial filter,
followed by a separate RCAS sharpening pass. Ultra Quality/Quality/Balanced/Performance
use the 1.3x/1.5x/1.7x/2x scale ratios. Manual 50–100% rendering uses bilinear
scaling when FSR is off. Upscaling affects the procedural backdrop; text, musicians,
graphs and controls stay at native output resolution. FSR is optional and its extra
passes may cost more than native rendering for this lightweight scene. Source and
license are in `assets/fsr1.wgsl`, adapted from
[AMD's reference implementation](https://github.com/GPUOpen-Effects/FidelityFX-FSR/blob/master/ffx-fsr/ffx_fsr1.h).

Camera FOV, right-drag mouse sensitivity, exposure, glow, saturation, vignette,
grain, chromatic split, scanlines, grid, lasers, orbit particle count, animation
speed, camera shake and reduced motion affect the procedural scene immediately.
These are visual controls for SIM artwork. The renderer provides spatial FSR 1;
ray tracing, temporal FSR and frame generation are not implemented.

Audio has master/music/effects/interface buses and a separate live gain for every
desktop sound: Nightfall, drummer and SSD explosions, pedal, BT/FX rhythm hits,
laser knob, navigation, command submission, error alert and celebration. The
**Test** buttons audition individual effects through the same mixer. Start audio
with the soundtrack toggle; pausing that loop leaves effects active. Gains combine
multiplicatively, so a master or bus mute silences its sounds. Closing the app
resets and closes the WinMM output. Apple Music and separately launched WAV/rips
players retain their own volume controls. The soundtrack is a single embedded
mix; individual instrument stems are not exposed.

**Save settings** writes bounded preferences to
`%LOCALAPPDATA%/opsec-demon/graphics-audio.json`; reset restores the live defaults.
No credentials are stored there. The smoke uses clean defaults and never writes
your preferences.

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
- **Audio:** the original Nightfall score loops automatically through the WinMM
  PCM mixer, with independent sound and bus gains. The sidebar toggles music;
  closing the app stops all audio.
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
adapter and a default Windows audio output. It checks real command output and
shared model effects, completes six real audio buffers with the master muted,
then renders all eight pages offscreen at 3840 x 2160. It writes
`target/desktop-4k.png`, `target/desktop-page-1-4k.png` through
`target/desktop-page-7-4k.png`, and `target/desktop-audio-source-4k.png`.
Four FSR presets and 50% bilinear scaling render into
`target/desktop-upscaler-1-4k.png` through `target/desktop-upscaler-5-4k.png`.
The report is `target/vulkan-smoke.json`. It then
opens the native Vulkan window for 120 updates and closes it automatically.
The report includes adapter identity and a cached-UI
render-throughput experiment without display/VSync or PNG encoding; it is not
an interactive FPS claim. Audio playback is muted during the smoke.

Hosted Windows CI runs native command smoke with explicit `-SkipVulkan`, since
it does not provide the required Vulkan adapter; local full smoke validates GPU
rendering. Packaging writes the standalone executable and a SHA-256 checksum
to `target/dist`, with the AMD shader license; build output remains ignored by Git.

To retain a terminal session:

```powershell
.\target\dist\OPSEC-DEMON.exe --shell
.\target\dist\OPSEC-DEMON.exe --plain -c 'echo hello | findstr hello'
```

For a machine with the pinned Rust target already installed, the equivalent
build is `powershell.exe -NoProfile -ExecutionPolicy Bypass -File
scripts/rust-windows.ps1 build --release --locked`.
