#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
for tool in rojo selene stylua lune; do
    command -v "$tool" >/dev/null || { printf '%s missing; run rokit install\n' "$tool" >&2; exit 1; }
done
stylua --check roblox
selene roblox/src
lune run roblox/tools/briefing.luau
lune run roblox/tools/session-tests.luau
mkdir -p roblox/build
rojo build default.project.json --output roblox/build/nightfall.rbxlx
