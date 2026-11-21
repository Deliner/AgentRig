# State

## Focus

P174, B181: make the gate faster and the rig code simpler without changing behavior. VAC 4 lets every hex SHA-256
of bytes go through the existing owner `shared::atomic_files::digest`.

## Workspace

Branch: feature/faster-gate
Revision: 6818a9c652d70f58519975fb5a90ea98cf21fdb6

## Progress

VAC 2 batched oracle discovery and VAC 3 batched committed blob reads. VAC 4 removes two copies of the same
computation: the reminder session hash in agent_hooks/reminder.rs and the two index digests in
check_evidence/input_fingerprint.rs now call `digest`, which before served only its own test. Values are unchanged.

## Verification

`just test agent_hooks::tests::reminders` passed 4, `just test check_evidence` 5 and `just test shared::atomic_files`
6. `just lint`, `just run rustfmt` and `just run clippy` are clean.

## Blockers

None.

## Next action

Commit VAC 4, then run the full test suite and the longest groups against the 103 s baseline of B181.
