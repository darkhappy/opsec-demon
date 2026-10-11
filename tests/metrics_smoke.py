"""Exercise the real standalone HTTP contract over a loopback socket."""
from pathlib import Path
import json
import socket
import subprocess
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
process = subprocess.Popen([str(ROOT/'target/release/opsec-demon'), '--metrics-server', '127.0.0.1:0'], stderr=subprocess.PIPE, text=True)
try:
    line = process.stderr.readline().strip()
    assert 'exporter listening on 127.0.0.1:' in line, line
    port = int(line.rsplit(':', 1)[1])
    base = f'http://127.0.0.1:{port}'
    for path in ['/healthz', '/readyz']:
        with urllib.request.urlopen(base+path, timeout=4) as response:
            assert response.status == 200 and response.read() == b'ok\n'
    with urllib.request.urlopen(base+'/metrics', timeout=4) as response:
        text = response.read().decode()
        assert 'opsec_demon_exporter_requests_total 3\n' in text
        assert 'opsec_demon_exporter_uptime_seconds' in text
    for path, method, code in [('/missing','GET',404),('/metrics','POST',405)]:
        try:
            urllib.request.urlopen(urllib.request.Request(base+path,method=method),timeout=4)
            raise AssertionError('Unexpected success')
        except urllib.error.HTTPError as error:
            assert error.code == code
    with socket.create_connection(('127.0.0.1',port),timeout=4) as client:
        client.sendall(b'NOT HTTP\r\n\r\n')
        assert b'400 Bad Request' in client.recv(4096)
    with urllib.request.urlopen(base+'/healthz',timeout=4) as response:
        assert response.status == 200
    print('Exporter: health, readiness, Prometheus values, methods, unknown paths, malformed clients, and survival passed.')
finally:
    process.terminate()
    process.wait(timeout=4)
