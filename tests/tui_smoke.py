"""Portable Linux/macOS PTY checks for the interactive command deck."""
import fcntl
import os
import pty
import re
import select
import struct
import subprocess
import sys
import tempfile
import termios
import time
from pathlib import Path

BINARY = Path(__file__).resolve().parents[1] / 'target/release/opsec-demon'
CSI = re.compile(rb'\x1b\[[0-?]*[ -/]*[@-~]')


class Session:
    def __init__(self, arguments, environment=None, size=(44, 140)):
        self.master, self.slave = pty.openpty()
        self.resize(*size)
        # Values above nine expose macOS stty's hex-save/decimal-restore mismatch.
        attributes = termios.tcgetattr(self.slave)
        attributes[6][termios.VMIN] = 16
        attributes[6][termios.VTIME] = 10
        termios.tcsetattr(self.slave, termios.TCSANOW, attributes)
        self.saved = termios.tcgetattr(self.slave)
        env = dict(os.environ, TERM='xterm-256color')
        env.pop('NO_COLOR', None)
        if environment:
            env.update(environment)
        self.process = subprocess.Popen([str(BINARY), *arguments], stdin=self.slave,
                                        stdout=self.slave, stderr=self.slave, env=env)
        self.started = time.monotonic()

    def resize(self, rows, columns):
        fcntl.ioctl(self.slave, termios.TIOCSWINSZ, struct.pack('HHHH', rows, columns, 0, 0))

    def send(self, value):
        os.write(self.master, value)

    def expect(self, text, timeout=4, raw=False):
        output = bytearray()
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if select.select([self.master], [], [], .05)[0]:
                output.extend(os.read(self.master, 65536))
                plain = CSI.sub(b'', output)
                if text in (output if raw else plain):
                    return bytes(output), plain
        raise AssertionError((text, CSI.sub(b'', output)[-4000:]))

    def finish(self):
        self.send(b'\x04')
        deadline = time.monotonic() + 4
        while self.process.poll() is None and time.monotonic() < deadline:
            if select.select([self.master], [], [], .05)[0]:
                try: os.read(self.master, 65536)
                except OSError: break
        self.process.wait(timeout=1)
        assert self.process.returncode == 0, self.process.returncode
        assert termios.tcgetattr(self.slave) == self.saved, 'terminal mode not restored'

    def close(self):
        if self.process.poll() is None:
            self.process.kill()
            self.process.wait()
        termios.tcsetattr(self.slave, termios.TCSANOW, self.saved)
        os.close(self.master)
        os.close(self.slave)


def dashboard():
    with tempfile.TemporaryDirectory(prefix='opsec-player-') as directory:
        for adapter in ['playerctl', 'osascript']:
            path = Path(directory) / adapter
            path.write_text("#!/bin/sh\nprintf 'Playing | Test Artist - Test Track\\n'\n")
            path.chmod(0o700)
        for adapter, script in {
                'aplay': "#!/bin/sh\nprintf 'audio-fixture\\n'\n",
                'afplay': "#!/bin/sh\nprintf 'audio-fixture\\n'\n",
                'starship': "#!/bin/sh\nprintf '\\033[32mSTARSHIP_FIXTURE\\033[0m'\n",
                'kubectl': "#!/bin/sh\nprintf '%s\\n' \"${KUBECONFIG:-demon-k8s}\"\n",
                'native-shell': "#!/bin/sh\nprintf 'native-shell-ok\\n'\n"}.items():
            path = Path(directory) / adapter
            path.write_text(script)
            path.chmod(0o700)
        session = Session(['--instant', '--mo-krill', '--egg'],
                          {'PATH': directory + os.pathsep + os.environ['PATH'], 'SHELL': directory + '/native-shell'})
        try:
            raw, plain = session.expect(b'EXECUTE >')
            assert b'\x1b[?1049h' in raw
            assert b'38;5;' in raw
            for label in [b'MO & KRILL', b'FREQUENCY RESPONSE', b'PHASE RESPONSE',
                          b'GROUP DELAY', b'EXCESS GROUP DELAY', b'MATRIX', b'[SIM] $',
                          b'BEATLES / ASCII TRIBUTE']:
                assert label in plain, label
            session.expect(b'Test Artist - Test Track')
            session.send(b'\x10')
            session.expect(b'ECHO VERIFIED //')
            session.expect(b'STARSHIP_FIXTURE')
            session.expect(b'demon-k8s')
            session.send(b'callsig\t nightwraith\r')
            session.expect(b'[IDENTITY] NIGHTWRAITH')
            session.send(b'export KUBECONFIG=bido-cluster\r')
            session.expect(b'[k8s: bido-cluster]')
            session.send(b'export OPSEC_TEST=environment-ok\r')
            session.send(b'printf "%s\\n" "$OPSEC_TEST"\r')
            session.expect('│ environment-ok'.encode())
            session.expect(b'COMMAND TRANSCRIPT')
            session.send(b'\x1b[A')
            session.expect(b'EXECUTE > printf')
            session.send(b'\x1b[B')
            session.send(b"printf 'signal acquired\\n' | cat\r")
            session.expect('│ signal acquired'.encode())
            session.expect(b'COMMAND TRANSCRIPT')
            session.send(b'callsign A\x1b[DB\r')
            session.expect(b'[IDENTITY] BA')
            session.send(b'graph metrics\r')
            session.expect(b'METRICS / ACTUAL SESSION')
            session.send(b'novel start\r')
            session.expect(b'VISUAL NOVEL')
            session.expect(b'THE STATION WITHOUT A NAME')
            session.send(b'psyche\r')
            session.expect(b'FICTIONAL CHARACTER ANALYSIS')
            session.send(b'mascot tweeter\r')
            session.expect(b'TWEETER / SPEAKER DEMON')
            session.send(b'mascot krillion\r')
            session.expect(b'KRILLION / SIGNAL SWARM')
            session.send(b'soundtrack play assets/audio/nightfall.wav\r')
            session.expect(b'audio-fixture')
            session.send(b'graph next\r')
            session.send(b'mascot mo-krill\r')
            session.send(b'music demo\r')
            session.expect(b'DEMO MUSIC')
            for command, expected in [
                    (b'bido\r', b'LITTLE BIDO'),
                    (b'bucket ls\r', b'club-penguin'),
                    (b'yoggies all\r', b'YOGGIE 5 / creamy shell'),
                    (b'yoggies use 3\r', b'YOGGIE 3 / NOM'),
                    (b'yoggies status\r', b'[OO.OO] 4/5 ready'),
                    (b'yoggies refill\r', b'all five yoggies restored'),
                    (b'tailscale milk\r', b'TAILSCALE MILK'),
                    (b'expansion expand\r', b'512 GB virtual'),
                    (b'expansion allocate 128\r', b'128 / 512'),
                    (b'mode prostreet\r', b'PROSTREET'),
                    (b'mode vulcan\r', b'VULCAN Z'),
                    (b'mode demon\r', b'[SIM SCENE]'),
                    (b'mascot chinese-demon\r', b'DEMON GATE / ASCII MASK'),
                    (b'icons on\r', b'[ICONS]'),
                    (b'doctor\r', b'[TERMINAL]')]:
                session.send(command)
                session.expect(expected)
            for command, expected in [
                    (b'ssd status\r', b'SIM FICTIONAL ODDS'),
                    (b'rt60 set 5 4 3 0.2\r', b'Sabine 0.514s'),
                    (b'graph room\r', b'RT60 / ROOM DECAY'),
                    (b'focus calculate\r', b'ceil(20/512) = 1'),
                    (b'coins generate\r', b'$1000 GENERATOR'),
                    (b'coins mint\r', b'minted 1 $POOPSECDEMONLARP'),
                    (b'sdvx play\r', b'[SIM SDVX PLAY]')]:
                session.send(command)
                session.expect(expected)
            session.send(b'asdfjkqeuo')
            session.expect(b'combo 6')
            session.expect(b'CONFETTI + FIREWORKS')
            session.send(b'\r')
            session.expect(b'controller released')
            for command, expected in [
                    (b'band drummer explode\r', b'drummer exploded'),
                    (b'headphones select hd58x\r', b'Sennheiser x Massdrop'),
                    (b'graph headphones\r', b'HEADPHONE EQ / UNMEASURED'),
                    (b'hoi4 select PHI\r', b'PHI | Philippines'),
                    (b'hoi4 focus NEON ARCHIPELAGO\r', b'NEON ARCHIPELAGO')]:
                session.send(command)
                session.expect(expected)
            session.send(b'\x07')
            session.expect(b'stomp accepted')
            session.send(b'graph next\r')
            session.send(b'mode demon\r')
            session.expect(b'[SIM SCENE]')
            session.send(b'ssd explode\r')
            session.expect(b'CONSTRUCT BREACHED')
            session.expect(b'imaginary debris settled', 5)
            session.send(b'egg hatch\r')
            session.expect(b'HATCH CHANNEL [SIM]')
            session.expect(b'EGG HATCHED [SIM]', 4)
            session.send(b'ritual\r')
            session.expect(b'CURSED APPLE [SIM / FICTIONAL]')
            session.send(b'\x18')
            session.expect(b'PARTICLE DETONATION')
            session.send(b'\x02')
            session.expect(b'POOPSEC / NIGHTFALL')
            session.send(b'\x02')
            session.expect(b'OPSEC / NIGHTFALL')
            session.send(b'sleep 5\r')
            session.expect(b'COMMAND / RUNNING')
            session.send(b'\x03')
            session.expect(b'[RETURN] exit 130', 3)
            session.send(b'shell\r')
            _, plain = session.expect(b'Press Enter to return')
            assert b'native-shell-ok' in plain
            session.send(b'\r')
            session.expect(b'EXECUTE >')
            session.send(b'shell printf fullscreen-ok\r')
            _, plain = session.expect(b'Press Enter to return')
            assert b'fullscreen-ok' in plain
            session.send(b'\r')
            session.expect(b'EXECUTE >')
            session.resize(24, 80)
            session.expect(b'EXECUTE >', 3)
            session.send(b'\x06')
            session.expect(b'LOST SIGNALS / VISUAL NOVEL')
            session.send(b'\x06')
            session.expect(b'[PANEL] FREQUENCY RESPONSE')
            session.send(b'\x06')
            session.expect(b'[PANEL] PHASE RESPONSE')
            session.resize(8, 20)
            session.expect(b'EXECUTE >', 3)
            session.finish()
        finally:
            session.close()
    print('PTY dashboard: inline commands, cursor editing, metadata, egg hatch, graphs, FX, cancellation, resize, terminal restoration passed.')



def redraws():
    with tempfile.TemporaryDirectory(prefix='opsec-wide-prompt-') as directory:
        player = Path(directory) / 'starship'
        player.write_text("#!/bin/sh\nprintf '%s' 'WIDE_PROMPT 界界界 🎵🎵🎵'\n")
        player.chmod(0o700)
        session = Session(['--instant'], {'PATH': directory + os.pathsep + os.environ['PATH']})
        try:
            session.expect('WIDE_PROMPT 界界界'.encode())
            for size in [(44, 140), (8, 20), (24, 80), (44, 140)]:
                session.resize(*size)
                output = bytearray()
                deadline = time.monotonic() + .3
                while time.monotonic() < deadline:
                    if select.select([session.master], [], [], .03)[0]:
                        output.extend(os.read(session.master, 65536))
                frames = re.findall(rb'\x1b\[\?2026h(.*?)\x1b\[\?2026l', output, re.S)
                assert frames, 'Missing complete synchronized redraw'
                for frame in frames:
                    # Even a stale oversized frame during resize must not scroll.
                    assert b'\n' not in frame and b'\r' not in frame
                    assert frame.startswith(b'\x1b[?7l\x1b[H')
                    assert frame.endswith(b'\x1b[?7h')
                    assert b'\x1b[1;1H' in frame
            session.finish()
        finally:
            session.close()
    print('PTY redraws: wide prompt, rapid shrink/grow, synchronized frames, and no scrolling line feeds passed.')


def boot():
    session = Session(['--mo-krill'])
    try:
        _, first = session.expect(b'[SIM] $')
        assert b'MATRIX ENTRY' in first
        session.expect(b'SENSOR / WAITING')
        assert b'EXECUTE >' not in first
        session.expect(b'FREQUENCY RESPONSE', 8)
        session.expect(b'EXCESS GROUP DELAY', 7)
        session.expect(b'48;5;255', 7, raw=True)
        session.expect(b'CONSTRUCT BREACHED', 2)
        _, plain = session.expect(b"SAY IT OUT LOUD: I'M IN.", 18)
        elapsed = time.monotonic() - session.started
        assert 15 <= elapsed < 17, elapsed
        assert b'CONNECTION ESTABLISHED' in plain
        session.finish()
        print('Boot ceremony: %.2fs with randomized simulation commands and deferred handshake.' % elapsed)
    finally:
        session.close()


def scripts():
    for arguments, expected in [(['-c', 'false'], 1), (['--tui'], 1),
                                (['--tui', '--shell'], 1), (['--egg', '-c', 'egg status'], 0)]:
        result = subprocess.run([str(BINARY), *arguments], stdin=subprocess.DEVNULL, capture_output=True)
        assert result.returncode == expected, (arguments, result)
    result = subprocess.run([str(BINARY)], input='mainframe local\nmainframe ping\nexit 0\n',
                            text=True, capture_output=True)
    assert result.returncode == 0
    assert 'ECHO VERIFIED' in result.stdout
    assert '\x1b' not in result.stdout
    payload = '{"events":["vectis prime -mod 20 impact", "lex prime -mod 20 puncture"],"mode":[1,2,2],"target":"-ehp 195 125 175"}'
    result = subprocess.run([str(BINARY), '-c', '..advancedttk ' + payload], text=True, capture_output=True)
    assert result.returncode == 0 and 'UNCOMPUTED' in result.stdout
    payload = payload[:-1] + ',"calculation":"individual-body"}'
    result = subprocess.run([str(BINARY), '-c', '..advancedttk ' + payload], text=True, capture_output=True)
    assert result.returncode == 0 and 'ESTIMATE | vectis prime' in result.stdout and 'ESTIMATE | lex prime' in result.stdout
    payload = payload.replace('individual-body', 'sequence-body')[:-1] + ',"swap_ms":500}'
    result = subprocess.run([str(BINARY), '-c', '..advancedttk ' + payload], text=True, capture_output=True)
    assert result.returncode == 0 and 'SHOT 2 | weapon 2 lex prime' in result.stdout
    assert 'target survives all 3 shots' in result.stdout
    print('Script mode: quiet output, simulator, exit status, egg CLI and Conclave JSON passed.')


if __name__ == '__main__':
    dashboard()
    redraws()
    scripts()
    if '--boot' in sys.argv:
        boot()
