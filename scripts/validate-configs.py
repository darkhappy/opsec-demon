"""Dependency-free shape checks; optional PyYAML covers non-template YAML."""
from pathlib import Path
import json
import re
import tomllib
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
SKIP = {'.git', 'target', 'node_modules', '.aws', '.agents', '__pycache__', 'Packages'}
try:
    import yaml
except ImportError:
    yaml = None

checked = 0
for path in ROOT.rglob('*'):
    relative = path.relative_to(ROOT)
    if not path.is_file() or any(part in SKIP for part in relative.parts):
        continue
    if path.suffix in {'.json', '.code-workspace', '.sublime-project'} or path.name in {'.prettierrc', '.luaurc'}:
        json.loads(path.read_text())
        checked += 1
    elif path.suffix == '.toml' or path.name == '.prettierrc.toml':
        tomllib.loads(path.read_text())
        checked += 1
    elif path.suffix in {'.xml', '.svg'}:
        ET.parse(path)
        checked += 1
    elif path.suffix in {'.yml', '.yaml'} and 'templates' not in relative.parts and yaml:
        list(yaml.safe_load_all(path.read_text()))
        checked += 1
    elif path.suffix == '.md':
        for target in re.findall(r'\[[^\]\n]+\]\(([^)]+)\)', path.read_text()):
            if target.startswith(('https:', 'http:', '#', 'mailto:')):
                continue
            target = target.split('#')[0]
            if target:
                assert (path.parent / target).exists(), f'{relative}: missing {target}'
        checked += 1
project = json.loads((ROOT / 'default.project.json').read_text())
def mounts(node):
    if not isinstance(node, dict):
        return
    if '$path' in node:
        assert (ROOT / node['$path']).exists(), node['$path']
    for value in node.values():
        mounts(value)
mounts(project['tree'])
print(f'Validated {checked} configuration/document files; YAML parser {"enabled" if yaml else "optional/not installed"}.')
print('Provider schemas and Helm templates require their native validators; this checks file structure and local references.')
