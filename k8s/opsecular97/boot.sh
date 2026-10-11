#!/bin/sh
set -eu
# Print a bounded fictional OS demonstration, then run the real local exporter.
opsec-demon --plain --no-banner --shell <<'CONSOLE'
opsecular boot
opsecular operator krillion
opsecular spirit 97
opsecular desktop
opsecular wsl
opsecular terminal
opsecular recycle put abandoned-lane.prop
opsecular recycle ls
rips list
opsecular poutine next
opsecular poutine next
opsecular poutine next
opsecular poutine next
opsecular poutine next
opsecular weep 3
exit
CONSOLE
exec opsec-demon --metrics-server 0.0.0.0:9090
