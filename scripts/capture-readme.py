"""Capture actual ANSI terminal frames as SVG using deterministic prompt fixtures."""
from pathlib import Path
from html import escape
import os
import re
import select
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tests'))
from tui_smoke import Session  # noqa: E402

CSI = re.compile(r'(\x1b\[[0-?]*[ -/]*[@-~])')

def rgb(index):
    base=['#000000','#800000','#008000','#808000','#000080','#800080','#008080','#c0c0c0','#808080','#ff0000','#00ff00','#ffff00','#0000ff','#ff00ff','#00ffff','#ffffff']
    if index < 16:
        return base[index]
    if index >= 232:
        n=8+(index-232)*10
        return f'#{n:02x}{n:02x}{n:02x}'
    n=index-16
    palette=[0,95,135,175,215,255]
    return '#'+''.join(f'{palette[v]:02x}' for v in [n//36,(n//6)%6,n%6])

def svg(frame, title):
    rows=frame.split('\r\n')[:44]
    result=[f'<svg xmlns="http://www.w3.org/2000/svg" width="1140" height="752" viewBox="0 0 1140 752" role="img" aria-labelledby="title"><title id="title">{escape(title)}</title>', '<rect width="1140" height="752" rx="12" fill="#080c13"/>', f'<text x="16" y="24" font-family="monospace" font-size="12" fill="#a5b7c9">{escape(title)} / ACTUAL ANSI FRAME / DEMO PROMPT + MUSIC</text>']
    fg='#d6dde7';bg=None
    for row,line in enumerate(rows):
        column=0
        for piece in CSI.split(line):
            if piece.startswith('\x1b['):
                if piece.endswith('m'):
                    parts=[int(x or 0) for x in piece[2:-1].split(';')]
                    i=0
                    while i<len(parts):
                        if parts[i]==0: fg='#d6dde7';bg=None
                        elif parts[i] in (38,48) and parts[i+1:i+2]==[5] and i+2<len(parts):
                            if parts[i]==38: fg=rgb(parts[i+2])
                            else: bg=rgb(parts[i+2])
                            i+=2
                        elif 30<=parts[i]<=37: fg=rgb(parts[i]-30)
                        elif 90<=parts[i]<=97: fg=rgb(parts[i]-90+8)
                        i+=1
                continue
            piece=''.join(c for c in piece if not c.iscontrol()) if hasattr(str,'iscontrol') else ''.join(c for c in piece if c.isprintable())
            if not piece: continue
            x=10+column*8;y=48+row*16
            if bg:
                result.append(f'<rect x="{x}" y="{y-12}" width="{len(piece)*8}" height="16" fill="{bg}"/>')
            result.append(f'<text x="{x}" y="{y}" xml:space="preserve" font-family="monospace" font-size="13" textLength="{len(piece)*8}" lengthAdjust="spacingAndGlyphs" fill="{fg}">{escape(piece)}</text>')
            column+=len(piece)
    return '\n'.join(result+['</svg>'])+'\n'

with tempfile.TemporaryDirectory(prefix='nightfall-capture-') as fixtures:
    for name, text in [('starship','nightfall'),('kubectl','demo-context')]:
        path=Path(fixtures)/name
        path.write_text(f"#!/bin/sh\nprintf '%s\\n' '{text}'\n")
        path.chmod(0o700)
    for name, commands in [
        ('terminal-metrics',['callsign NIGHTWRAITH','theme venom','music demo','mascot tweeter','graph metrics']),
        ('terminal-novel',['callsign NIGHTWRAITH','theme ghost','music demo','mascot krillion','novel start']),
        ('terminal-theater',['callsign NIGHTWRAITH','theme crimson','music demo','mascot tweeter','graph next']),
    ]:
        session=Session(['--instant'],{'PATH': fixtures+os.pathsep+os.environ['PATH']}, size=(44,140))
        try:
            session.expect(b'EXECUTE >')
            for command in commands:
                session.send(command.encode()+b'\r')
                end=time.monotonic()+.15
                while time.monotonic()<end:
                    if select.select([session.master],[],[],.03)[0]: os.read(session.master,65536)
            output=bytearray();end=time.monotonic()+.4
            while time.monotonic()<end:
                if select.select([session.master],[],[],.03)[0]: output.extend(os.read(session.master,65536))
            frames=bytes(output).split(b'\x1b[H')
            assert len(frames)>=3, 'Need complete redraws'
            frame=frames[-2].decode('utf-8',errors='replace')
            path=ROOT/'docs/assets'/f'{name}.svg'
            path.write_text(svg(frame,name.upper()))
            print(path.relative_to(ROOT))
            session.finish()
        finally:
            session.close()
