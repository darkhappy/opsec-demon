#!/bin/sh
set -eu

# Pinned official Nerd Fonts release; SHA-256 from the published release assets.
font_version=3.5.1
font_sha=04d5e8f903693f9dd13e16f867e994834e681eb3c72c0d337a770dcda09010cf
font_workspace=$(mktemp -d "${TMPDIR:-/tmp}/opsec-font.XXXXXX")
trap 'rm -rf "$font_workspace"' EXIT HUP INT TERM
font_archive="$font_workspace/JetBrainsMono.tar.xz"
printf '[1/5] LOADING FONT ARCHIVE / official Nerd Fonts %s / network download\n' "$font_version"
curl --progress-bar --fail --location --connect-timeout 10 --max-time 120 \
    "https://github.com/ryanoasis/nerd-fonts/releases/download/v$font_version/JetBrainsMono.tar.xz" \
    --output "$font_archive"
printf '[2/5] READING ARCHIVE / verifying published SHA-256\n'
if command -v shasum >/dev/null 2>&1; then
    font_actual=$(shasum -a 256 "$font_archive" | awk '{print $1}')
else
    font_actual=$(sha256sum "$font_archive" | awk '{print $1}')
fi
if [ "$font_actual" != "$font_sha" ]; then
    echo 'Font archive checksum mismatch.' >&2
    exit 1
fi
printf '[3/5] DECODING GLYPH BANK / extracting archive\n'
mkdir "$font_workspace/extracted"
tar -xJf "$font_archive" -C "$font_workspace/extracted"
case $(uname -s) in
    Darwin) font_destination="$HOME/Library/Fonts" ;;
    Linux) font_destination="${XDG_DATA_HOME:-$HOME/.local/share}/fonts/opsec-demon" ;;
    *) echo 'Font installation supports macOS and Linux.' >&2; exit 1 ;;
esac
font_destination=${OPSEC_FONT_DIR:-$font_destination}
mkdir -p "$font_destination"
printf '[4/5] INSTALLING NERD GLYPHS / destination %s\n' "$font_destination"
font_count=0
for font_file in "$font_workspace/extracted"/JetBrainsMonoNerdFontMono-*.ttf; do
    if [ -f "$font_file" ]; then
        printf '[READ] %s\n' "${font_file##*/}"
        cp "$font_file" "$font_destination/"
        font_count=$((font_count + 1))
    fi
done
if [ "$font_count" -eq 0 ]; then
    echo 'The archive did not contain JetBrainsMono Nerd Font Mono fonts.' >&2
    exit 1
fi
printf '[5/5] REBUILDING FONT CACHE / finishing local installation\n'
if [ "$(uname -s)" = Linux ] && command -v fc-cache >/dev/null 2>&1; then
    fc-cache -f "$font_destination"
fi
printf 'Installed %s font files in %s\n' "$font_count" "$font_destination"
printf 'Select JetBrainsMono Nerd Font Mono in your terminal, then launch with --nerd-font.\n'
