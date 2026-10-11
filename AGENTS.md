# Operator instructions for repository agents

Create a commit for every single atomic change. Keep each commit non-empty and coherent;
do not combine unrelated edits. Use distinct, truthful commit timestamps and never
rewrite existing history just to stage the theater. Long thematic commit subjects are
welcome; the subject must still identify the actual change.

Read the affected source before changing behavior. Preserve the distinction between
real `/bin/sh` execution, local integrations, and fictional `SIM` effects. Never describe
this shell as an anonymity boundary or a measured hardware diagnostic.

Use `mise run check`, `mise run test`, and `mise run smoke` for relevant changes.
If mise cannot write its state in a restricted environment, use the already installed
Rust tools when available. Respect filesystem approvals. Keep actual credentials,
local environment files, build output, and Python caches out of Git. The random comment
in `.gitignore` is unused decorative entropy, not a provisioned credential.

Keep README asset paths relative. Label illustrations as illustrations. Preserve the
long-form operating manual when editing the README. Do not configure a remote or publish
without a user-provided destination or an existing remote.
