# State

## Focus

P151 (B152) hook checks every tool call, VAC2 for AC1 and AC2: both agents call the hook for every tool, which checks
shell commands and file edits, passes listed passive tools and refuses the rest by name; a hook that cannot start
exits 2 with its reason (D200).

## Workspace

Branch: feature/hook-every-tool
Revision: 28cf511523aea2738c459d31e176cc0d77bca654

## Progress

VAC1 (AC3, D199) is committed. VAC2 adds `agent_hooks/tools.rs`, uses it in dispatch and configuration recovery, reads
`notebook_path` as an edit path, registers PreToolUse with matcher `*` and replaces `exec` in the hook command with a
fallback that exits 2. The rig's `.claude/settings.json` and `.codex/hooks.json` match the init output.

## Verification

`just run test -- agent_hooks:: project_setup::` passed 88 tests, including the new Monitor, NotebookEdit, refused,
passive and recovery tests and the registration test with a non-executable and a missing launcher. Candidate lint,
Clippy and the memory check pass.

## Blockers

None.

## Next action

Commit VAC2, pin it with `just bootstrap HEAD`, probe Claude Code live, then archive P151 and B152 and merge.
