# Nightfall / Tweeter & Krillion original score

## OpSecULAR 97 remix-style album

`rips siiva` prints nine credited listening links from the official
[SiIvaGunner KFAD MOJO!! music archive](https://mojo.highquality.rip/category/music/):
Lifelight, 8 PM (Fishing Night in Heaven), Main Menu, Hilltop Rave, Launch Base '93,
Funny Man Goes to the Tool Shop, NEO KOBE外伝, A Spacecraft Fueled on Fright, and
Reggie's Late Nite. Open the pages in a browser to listen using their hosted players.
The catalog is external; the shell does not download or bundle those recordings.

`rips list` lists nine original 16-bar synth songs with different tempos, keys,
harmonics, and surprise melody switches. This is an unaffiliated tribute to
SiIvaGunner-style remix humor, with no SiIvaGunner recordings or sampled game music.
The tracks are synthesized on demand as 22.05 kHz, 16-bit stereo PCM WAV files:

```text
rips render 1 /tmp/recycle-bin.wav
soundtrack play /tmp/recycle-bin.wav
```

IDs 1–9 select Recycle Bin Bossa, WSL Jade Boot, Krillion Lane Counterpick,
Larp City Bus Ride, Lost Signals Afterparty, Curds of Destiny, Mainframe Tears,
Serial Clown Inferno, and OpSecULAR 97 Credits. Files are created exclusively;
existing paths are never overwritten. Playback uses the same host audio adapter
as the original Nightfall score. Rendering does not require an audio device.

The included soundtrack is an original 32-bar, 148 BPM E-minor synth track: kick, bass,
snare, arpeggio, pad, stereo echo, and fades. The PCM WAV is 44.1 kHz, 16-bit stereo,
51.89 seconds long. No music from Lost Child or another game is sampled.

```text
soundtrack status
soundtrack render /tmp/nightfall.wav
soundtrack play /tmp/nightfall.wav
dac status
dac serial
```

Render creates a new file and refuses to overwrite an existing path. The repository
copy is `assets/audio/nightfall.wav`; `soundtrack play` without a path uses that relative
file from the working directory. Playback uses `aplay` on Linux and `afplay` on macOS.
In the TUI it runs as a child job; Ctrl-C stops playback while the dashboard remains alive.
The binary embeds the score, so rendering also works outside the source checkout.

For the requested Focusrite Scarlett Solo 3, playback uses the OS USB audio stack.
On Linux, list actual device IDs with `shell aplay -L`, then select your visible card:

```text
export OPSEC_DEMON_AUDIO_DEVICE=plughw:CARD=YOUR_CARD,DEV=0
soundtrack play /tmp/nightfall.wav
```

Replace YOUR_CARD with your actual device ID; no fixed Scarlett identifier is assumed.
On macOS select the interface in system sound settings. WSL must have a working audio
route/device visible to its chosen player. No attached hardware was available to verify
physical output during repository testing. `dac serial` only lists serial device names;
it does not send UART commands or change gain, phantom power, or device firmware.

Recompose the score to a new output file with
`python3 scripts/compose-soundtrack.py --output /tmp/recomposed-nightfall.wav`.
