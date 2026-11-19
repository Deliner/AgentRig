# State

## Focus

P157 (B158), VAC 3: a decision marker counts only in a source file its decision links (AC2), D216.

## Workspace

Branch: feature/gate-base-rules
Revision: 74f93a3b7c4a1e0b6c6cb263bc1947b2b7f38270

## Progress

VAC 1 (D214) and VAC 2 (D215) committed. VAC 3: the memory check refuses a `DECISION:` marker in a source file that
the decision does not link under Applies in, naming each decision and file; `INVARIANT:` markers keep the existence
check. Eighteen decision rows gained links to the files that already carried their markers.

## Verification

`just run clippy`, `just run rustfmt`, `just lint` and `just candidate check --only memory` pass. `just run test`
passes 585 tests, including the E3 test of a marker over `unwrap` in an unlinked file, refused and then accepted.

## Blockers

None.

## Next action

Commit VAC 3, then VAC 4: AC6 the installed linter also checks a rig change (D217).
