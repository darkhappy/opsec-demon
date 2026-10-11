#!/usr/bin/env bash
# Run from a normal WSL terminal to install and start the per-user bridge.
set -euo pipefail
script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
command -v powershell.exe >/dev/null || { echo 'Windows PowerShell is unavailable in this WSL session.' >&2; exit 1; }
/usr/bin/python3 -c 'from gi.repository import Gio, GLib' || {
    echo 'Install dependencies first: sudo apt install python3-gi playerctl' >&2
    exit 1
}
command -v playerctl >/dev/null || { echo 'Install playerctl: sudo apt install playerctl' >&2; exit 1; }
# Check access before writing configuration.
systemctl --user show-environment >/dev/null || {
    echo 'No systemd user session. Run the bridge in a terminal instead:' >&2
    echo "  /usr/bin/python3 \"$script_dir/windows-music.py\"" >&2
    echo 'If D-Bus is unavailable, use dbus-run-session -- bash and run the bridge and your apps inside that shell.' >&2
    exit 1
}
config_dir="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
mkdir -p -- "$config_dir"
# systemd specifiers and quoting need escaping even inside a quoted argument.
unit_script=${script_dir//\\/\\\\}
unit_script=${unit_script//\"/\\\"}
unit_script=${unit_script//%/%%}
cat > "$config_dir/windows-music.service" <<EOF
[Unit]
Description=Windows Apple Music to WSL MPRIS bridge

[Service]
ExecStart=/usr/bin/python3 "$unit_script/windows-music.py"
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
EOF
systemctl --user daemon-reload
systemctl --user enable --now windows-music.service
# enable --now does not restart an already running service after script changes.
systemctl --user restart windows-music.service
echo 'Bridge enabled for your WSL user. Play a track in the Windows Apple Music app, then run:'
echo "  playerctl -p AppleMusicWindows metadata --format '{{status}} | {{artist}} - {{title}}'"
echo 'Logs: journalctl --user -u windows-music.service -n 30'
echo 'Disable: systemctl --user disable --now windows-music.service'
