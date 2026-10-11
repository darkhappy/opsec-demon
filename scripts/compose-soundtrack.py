"""Compose the original Nightfall / Tweeter & Krillion score using only stdlib."""
import argparse
import math
import random
import struct
import wave
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--output', default='assets/audio/nightfall.wav')
args = parser.parse_args()
output = Path(args.output)
output.parent.mkdir(parents=True, exist_ok=True)
rate = 44100
bpm = 148
beat = 60 / bpm
bars = 32
duration = bars * 4 * beat
rng = random.Random(999)
roots = [40, 36, 43, 38]  # E minor / C / G / D
arp = [0, 7, 12, 15, 19, 15, 12, 7]
delay = [0.0] * int(rate * beat * .75)
with output.open('xb') as raw:
    with wave.open(raw, 'wb') as track:
        track.setnchannels(2)
        track.setsampwidth(2)
        track.setframerate(rate)
        chunk = bytearray()
        for index in range(int(rate * duration)):
            t = index / rate
            beats = t / beat
            bar = int(beats / 4)
            step = int(beats * 4)
            local = t % beat
            subdivision = t % (beat / 4)
            root = roots[(bar // 2) % len(roots)]
            frequency = 440 * 2 ** ((root - 69) / 12)
            # Soft saturation and envelopes keep the output under full scale.
            bass_phase = t * frequency
            bass = math.tanh(2.2 * math.sin(math.tau * bass_phase)) * math.exp(-subdivision * 13)
            bass *= .16 if 4 <= bar < 28 else .07
            kick = math.sin(math.tau * (45 * local + 7 * (1 - math.exp(-local * 28))))
            kick *= math.exp(-local * 17) * (.34 if 4 <= bar < 28 else .12)
            noise = rng.uniform(-1, 1)
            snare = noise * math.exp(-local * 28) * (.14 if int(beats) % 4 in (1, 3) else 0)
            hat = noise * math.exp(-subdivision * 105) * (.04 if 4 <= bar < 28 else .015)
            lead_frequency = frequency * 2 ** ((24 + arp[step % 8]) / 12)
            lead = math.sin(math.tau * lead_frequency * t) + .25 * math.sin(math.tau * lead_frequency * t * 2)
            lead *= .075 * math.exp(-subdivision * 18) * min(1, max(0, (bar - 6) / 3))
            pad = sum(math.sin(math.tau * frequency * 2 ** (note / 12) * t) for note in (12, 15, 19)) * .018
            slot = index % len(delay)
            echo = delay[slot]
            delay[slot] = lead + echo * .38
            mix = bass + kick + snare + hat + lead + pad
            fade = min(1, t / 2, (duration - t) / 3)
            left = .66 * math.tanh((mix + echo * .42) * 1.1) * fade
            right = .66 * math.tanh((mix + echo * .58) * 1.1) * fade
            chunk.extend(struct.pack('<hh', round(left * 32767), round(right * 32767)))
            if len(chunk) >= 65536:
                track.writeframesraw(chunk)
                chunk.clear()
        track.writeframes(chunk)
print(f'Original score: {duration:.2f}s / {bpm} BPM / E minor / stereo {rate}Hz / {output}')
