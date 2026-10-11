# Nightfall / Tweeter & Krillion original score

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
