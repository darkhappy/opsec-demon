# Windows bridge authorization

Fictional filing authority: NIGHTFALL / ABYSS-999.

From WSL, `bash scripts/setup-windows-music.sh` sets up the Windows media-session to MPRIS bridge. The setup uses existing playerctl and python3-gi.

The bridge exposes metadata only, not playback controls, artwork, or position. Read the full README for service and D-Bus instructions.

See the [full operating manual](../../README.md) and [actual security boundaries](../../SECURITY.md).
