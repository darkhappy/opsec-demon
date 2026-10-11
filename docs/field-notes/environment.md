# Persistent local overrides

Fictional filing authority: NIGHTFALL / ABYSS-999.

Standalone `export NAME=value` persists a child-process override. `unset NAME` removes it. Compound commands run in their own child shell, so state created there does not persist in the deck.

The repository `.env.example` is documentation. The shell does not automatically source it.

See the [full operating manual](../../README.md) and [actual security boundaries](../../SECURITY.md).
