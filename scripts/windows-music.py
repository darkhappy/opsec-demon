#!/usr/bin/python3
"""Expose Windows Apple Music as a read-only MPRIS player in WSL."""
import argparse
import hashlib
import json
import os
import queue
import shutil
import signal
import subprocess
import sys
import threading
import time
from pathlib import Path

PLAYER = 'org.mpris.MediaPlayer2.Player'
ROOT = 'org.mpris.MediaPlayer2'
BUS_NAME = ROOT + '.AppleMusicWindows'
OBJECT_PATH = '/org/mpris/MediaPlayer2'


def read_windows():
    powershell = shutil.which('powershell.exe')
    if not powershell:
        # systemd's PATH normally omits the Windows directories from WSL's PATH.
        default = Path('/mnt/c/Windows/System32/WindowsPowerShell/v1.0/powershell.exe')
        if default.is_file():
            powershell = str(default)
    if not powershell:
        raise RuntimeError('powershell.exe is unavailable; enable Windows interop in WSL')
    environment = os.environ.copy()
    if not Path(environment.get('WSL_INTEROP', '/nonexistent')).is_socket():
        # User services do not necessarily inherit the invoking terminal's socket.
        sockets = [path for path in Path('/run/WSL').glob('*_interop') if path.is_socket()]
        if sockets:
            environment['WSL_INTEROP'] = str(max(sockets, key=lambda path: path.stat().st_mtime))
    script = Path(__file__).with_suffix('.ps1').read_text()
    # EncodedCommand avoids UNC execution-policy and path-quoting problems.
    import base64
    encoded = base64.b64encode(script.encode('utf-16le')).decode('ascii')
    return subprocess.Popen(
        [powershell, '-NoLogo', '-NoProfile', '-NonInteractive',
         '-InputFormat', 'Text', '-OutputFormat', 'Text',
         '-EncodedCommand', encoded], stdout=subprocess.PIPE,
        encoding='utf-8-sig', errors='replace', bufsize=1, env=environment,
    )


def properties(state, GLib):
    metadata = {}
    if state.get('source'):
        digest = hashlib.sha256(json.dumps(
            [state.get(key, '') for key in ('source', 'title', 'artist', 'album')]
        ).encode()).hexdigest()
        metadata = {
            'mpris:trackid': GLib.Variant('o', OBJECT_PATH + '/track/' + digest),
            'xesam:title': GLib.Variant('s', state.get('title') or ''),
            'xesam:artist': GLib.Variant('as', [state.get('artist') or '']),
            'xesam:album': GLib.Variant('s', state.get('album') or ''),
        }
    status = state.get('status', 'Stopped')
    if status not in ('Playing', 'Paused'):
        status = 'Stopped'
    result = {
        'PlaybackStatus': GLib.Variant('s', status),
        'Metadata': GLib.Variant('a{sv}', metadata),
        'Rate': GLib.Variant('d', 1.0),
        'MinimumRate': GLib.Variant('d', 1.0),
        'MaximumRate': GLib.Variant('d', 1.0),
        'Volume': GLib.Variant('d', 1.0),
        'Position': GLib.Variant('x', 0),
    }
    for name in ('CanControl', 'CanPlay', 'CanPause', 'CanSeek', 'CanGoNext', 'CanGoPrevious'):
        result[name] = GLib.Variant('b', False)
    return result


def serve(child):
    try:
        from gi.repository import Gio, GLib
    except ImportError as error:
        raise RuntimeError('Install python3-gi: sudo apt install python3-gi') from error
    bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
    reply = bus.call_sync(
        'org.freedesktop.DBus', '/org/freedesktop/DBus', 'org.freedesktop.DBus',
        'RequestName', GLib.Variant('(su)', (BUS_NAME, 4)),
        GLib.VariantType('(u)'), Gio.DBusCallFlags.NONE, 5000, None,
    )
    if reply.unpack()[0] != 1:
        raise RuntimeError('The AppleMusicWindows bridge is already running')
    root_properties = {
        'Identity': GLib.Variant('s', 'Apple Music (Windows)'),
        'CanQuit': GLib.Variant('b', False),
        'CanRaise': GLib.Variant('b', False),
        'HasTrackList': GLib.Variant('b', False),
        'SupportedUriSchemes': GLib.Variant('as', []),
        'SupportedMimeTypes': GLib.Variant('as', []),
    }
    current = properties({}, GLib)
    xml = '<node>'
    for interface, values in ((ROOT, root_properties), (PLAYER, current)):
        xml += f'<interface name="{interface}">'
        for key, value in values.items():
            xml += f'<property name="{key}" type="{value.get_type_string()}" access="read"/>'
        if interface == PLAYER:
            for method in ('Play', 'Pause', 'PlayPause', 'Stop', 'Next', 'Previous'):
                xml += f'<method name="{method}"/>'
        xml += '</interface>'
    node = Gio.DBusNodeInfo.new_for_xml(xml + '</node>')

    def get_property(_bus, _sender, _path, interface, name):
        return (root_properties if interface == ROOT else current).get(name)

    def method_call(_bus, _sender, _path, _interface, _method, _params, invocation):
        invocation.return_dbus_error('org.freedesktop.DBus.Error.NotSupported',
                                     'This bridge provides playback metadata only')

    for interface in node.interfaces:
        bus.register_object(OBJECT_PATH, interface, method_call, get_property, None)
    inbox = queue.Queue(maxsize=1)

    def reader():
        invalid_reported = False
        for line in child.stdout:
            try:
                state = json.loads(line)
                if not isinstance(state, dict):
                    continue
            except ValueError:
                if not invalid_reported:
                    print(f'Unexpected Windows reader output: {line[:300].rstrip()!r}',
                          file=sys.stderr, flush=True)
                    invalid_reported = True
                continue
            try:
                inbox.get_nowait()
            except queue.Empty:
                pass
            inbox.put(state)

    threading.Thread(target=reader, daemon=True).start()
    loop = GLib.MainLoop()
    last_update = time.monotonic()
    failed = False
    received = False
    last_source = None

    def refresh():
        nonlocal current, last_update, failed, received, last_source
        try:
            state = inbox.get_nowait()
            last_update = time.monotonic()
            received = True
            source = (state.get('source', ''), tuple(state.get('sessions', [])))
            if source != last_source:
                if source[0]:
                    print(f'Connected to Windows media session: {source[0]}',
                          file=sys.stderr, flush=True)
                else:
                    print(f'No Apple Music session found. Windows sessions: {list(source[1])}',
                          file=sys.stderr, flush=True)
                last_source = source
        except queue.Empty:
            state = {} if time.monotonic() - last_update > 8 else None
        if time.monotonic() - last_update > 15:
            print('Windows reader stopped sending JSON' if received else
                  'Windows reader sent no JSON; check PowerShell errors above',
                  file=sys.stderr, flush=True)
            failed = True
            loop.quit()
            return False
        if state is not None:
            updated = properties(state, GLib)
            changed = {key: value for key, value in updated.items() if value != current[key]}
            current = updated
            if changed:
                bus.emit_signal(None, OBJECT_PATH, 'org.freedesktop.DBus.Properties',
                                'PropertiesChanged', GLib.Variant('(sa{sv}as)',
                                                               (PLAYER, changed, [])))
        if child.poll() is not None:
            print('Windows reader exited; check WSL interop and Windows media API access',
                  file=sys.stderr)
            failed = True
            loop.quit()
            return False
        return True

    GLib.timeout_add(200, refresh)
    for sig in (signal.SIGINT, signal.SIGTERM):
        GLib.unix_signal_add(GLib.PRIORITY_DEFAULT, sig, lambda: (loop.quit(), False)[1])
    print('MPRIS registered; waiting for Windows playback data', file=sys.stderr, flush=True)
    loop.run()
    if failed:
        raise RuntimeError('Windows reader failed')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--json', action='store_true', help='stream JSON without D-Bus')
    args = parser.parse_args()
    child = None
    try:
        child = read_windows()
        if args.json:
            for line in child.stdout:
                print(line.rstrip(), flush=True)
            if child.wait() != 0:
                raise RuntimeError('Windows reader failed')
        else:
            serve(child)
    except KeyboardInterrupt:
        pass
    except Exception as error:
        print(f'windows-music: {error}', file=sys.stderr)
        return 1
    finally:
        if child is not None and child.poll() is None:
            child.terminate()
            try:
                child.wait(timeout=3)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait()
    return 0


if __name__ == '__main__':
    sys.exit(main())
