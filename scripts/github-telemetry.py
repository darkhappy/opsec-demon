"""Generate offline README charts from committed Git history and tracked filenames."""
from collections import Counter
from datetime import datetime, timezone
from html import escape
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def git(*args):
    return subprocess.check_output(['git', '-C', str(ROOT), *args], text=True).strip()

revision = git('rev-parse', 'HEAD')
rows = [line.split() for line in git('log', '--reverse', '--format=%h %ct').splitlines()]
files = git('ls-files', '-z').split('\0')
files = [name for name in files if name]
kinds = Counter(Path(name).suffix or '[extensionless]' for name in files)
report = {
    'basis_revision': revision,
    'generated_at_utc': datetime.now(timezone.utc).isoformat(),
    'scope': 'Local committed history and tracked filenames; no GitHub stars or hosted CI results.',
    'commits': len(rows), 'tracked_files': len(files),
    'first_commit_utc': datetime.fromtimestamp(int(rows[0][1]), timezone.utc).isoformat(),
    'last_commit_utc': datetime.fromtimestamp(int(rows[-1][1]), timezone.utc).isoformat(),
    'file_extensions': dict(kinds.most_common()),
}
(ROOT / 'docs/github/local-telemetry.json').write_text(json.dumps(report, indent=2) + '\n')


def text(x, y, value, color='#a5b7c9', size=15):
    return f'<text x="{x}" y="{y}" fill="{color}" font-family="monospace" font-size="{size}">{escape(str(value))}</text>'


def card(title, subtitle, content, filename):
    svg = ['<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="460" viewBox="0 0 1200 460" role="img" aria-labelledby="title desc">',
           f'<title id="title">{escape(title)}</title><desc id="desc">{escape(subtitle)}</desc>',
           '<rect width="1200" height="460" rx="20" fill="#080c13"/>',
           '<rect x="18" y="18" width="1164" height="424" rx="14" fill="none" stroke="#27354b"/>',
           text(40, 55, title, '#72ffad', 24), text(40, 84, subtitle, size=13), *content,
           text(40, 420, f'LOCAL SNAPSHOT / {revision[:12]} / regenerate: python3 scripts/github-telemetry.py', size=13), '</svg>']
    (ROOT / 'docs/assets' / filename).write_text('\n'.join(svg) + '\n')

first, last = int(rows[0][1]), int(rows[-1][1])
span = max(last-first, 1)
points = [(70 + (int(stamp)-first)/span*1050, 365-(index+1)/len(rows)*240) for index, (_, stamp) in enumerate(rows)]
line = ' '.join(f'{x:.1f},{y:.1f}' for x,y in points)
content = [f'<path d="M70 120V365H1120" stroke="#40516b" fill="none"/>',
           f'<polyline points="{line}" fill="none" stroke="#ff4265" stroke-width="3"/>',
           text(70, 390, report['first_commit_utc'][:16]+' UTC', size=12),
           text(850, 390, report['last_commit_utc'][:16]+' UTC', size=12),
           text(80, 135, f'{len(rows)} actual commits', '#ff4265', 20)]
card('COMMIT ASCENSION // DIRECTORATE PAPER TRAIL', 'Cumulative commits over real committer timestamps. Local history, not GitHub activity.', content, 'github-commit-history.svg')
content=[]
for i,(kind,count) in enumerate(kinds.most_common(8)):
    y=130+i*32
    content += [text(45,y+16,kind,'#72ffad'), f'<rect x="215" y="{y}" width="{count/kinds.most_common(1)[0][1]*770:.1f}" height="21" rx="4" fill="{["#ff4265","#72ffad","#51e0ff"][i%3]}"/>', text(1010,y+16,count)]
card('FILETYPE ARSENAL // TRACKED ARTIFACT CENSUS', f'{len(files)} tracked files. Top eight extension counts; filename counts, not source bytes or language expertise.', content, 'github-file-census.svg')
ci = ['GitHub','GitLab','Bitbucket','TeamCity','Jenkins','Azure','CircleCI','Buildkite','Drone','Woodpecker','Travis','Cirrus','Semaphore','AWS CodeBuild','Google Cloud Build','SourceHut','Codefresh','Concourse','Gitea','Forgejo']
content=[]
for i,name in enumerate(ci):
    x=40+(i%5)*226;y=115+(i//5)*65
    content += [f'<rect x="{x}" y="{y}" width="210" height="50" rx="8" fill="#111b2b" stroke="#344860"/>',text(x+12,y+30,name,'#51e0ff',14)]
content += [text(40,395,'20 adapter files → scripts/ci.sh → fmt / clippy / tests / build / PTY', '#ff4265',16)]
card('DEUS EX MACHINA // CI ROUTING CONSTELLATION', 'Configuration topology illustration. Adapter presence does not mean hosted builds have passed.', content, 'github-ci-constellation.svg')
print(f'Wrote telemetry and three charts for {revision[:12]}.')
