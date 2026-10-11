# Extraction and terminal recovery

Fictional filing authority: NIGHTFALL / ABYSS-999.

Ctrl-D exits and restores the terminal. Normal exits and handled errors restore input modes. If forcibly killed, `stty sane` can restore input.

The PTY suite checks restoration, cancellation, resizing, and cinematic behavior. A process cannot guarantee cleanup after an uncatchable kill.

See the [full operating manual](../../README.md) and [actual security boundaries](../../SECURITY.md).
