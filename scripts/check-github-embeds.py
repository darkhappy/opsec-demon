"""Explicit network probe of public README images; never runs as part of normal CI."""
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import json
from pathlib import Path
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parents[1]
manifest = json.loads((ROOT / 'docs/github/embeds.json').read_text())


def probe(item):
    name, url = item
    result = {'name': name, 'url': url}
    try:
        request = Request(url, headers={'User-Agent': 'opsec-demon-readme-probe/1.0'})
        with urlopen(request, timeout=15) as response:
            body = response.read(65536)
            result.update(status=response.status,
                          content_type=response.headers.get('Content-Type', ''),
                          svg_markup_seen=b'<svg' in body,
                          checked_bytes=len(body))
    except HTTPError as error:
        result.update(status=error.code, error='HTTP error')
    except (URLError, TimeoutError, OSError) as error:
        result.update(status=None, error=str(error))
    return result


with ThreadPoolExecutor(max_workers=4) as executor:
    results = list(executor.map(probe, manifest['images'].items()))
report = {'checked_at_utc': datetime.now(timezone.utc).isoformat(),
          'scope': 'Public URL availability snapshot; HTTP 200 SVG can still contain a service error card.',
          'results': results}
(ROOT / 'docs/github/embed-probe.json').write_text(json.dumps(report, indent=2) + '\n')
for result in results:
    print(f"{result['name']}: HTTP {result['status']}; SVG markup {result.get('svg_markup_seen', False)}")
