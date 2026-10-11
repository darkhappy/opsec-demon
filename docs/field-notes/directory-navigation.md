# Working-directory orders

Fictional filing authority: NIGHTFALL / ABYSS-999.

Use `cd "directory with spaces"`, `cd -`, and `pwd`. Directory built-ins support quotes and `~/`; variable expansion is not implemented by the `cd` built-in.

The current directory affects subsequent commands and prompt helpers. A child-shell `cd` does not move the parent deck.

See the [full operating manual](../../README.md) and [actual security boundaries](../../SECURITY.md).
