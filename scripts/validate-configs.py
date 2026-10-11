"""Dependency-free shape checks; optional PyYAML covers non-template YAML."""
from pathlib import Path
import json
from html.parser import HTMLParser
import re
import tomllib
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
SKIP = {'.git', 'target', 'node_modules', '.aws', '.agents', '__pycache__', 'Packages'}
try:
    import yaml
except ImportError:
    yaml = None

class ImageReferences(HTMLParser):
    def __init__(self):
        super().__init__()
        self.sources = []

    def handle_starttag(self, tag, attrs):
        if tag.lower() == 'img':
            self.sources.extend(value for name, value in attrs if name == 'src' and value)


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
        document = path.read_text()
        images = ImageReferences()
        images.feed(document)
        targets = re.findall(r'\[[^\]\n]+\]\(([^)]+)\)', document) + images.sources
        for target in targets:
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
