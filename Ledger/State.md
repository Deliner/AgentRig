# State

## Focus

P157 (B158), VAC 2: refuse decision and invariant changes relative to the fork from the base (AC4, AC5), D215.

## Workspace

Branch: feature/gate-base-rules
Revision: 4c861c1f62b137dd9b507aabbe7289354a70ec4e

## Progress

VAC 1 committed (D214, AC1 and AC3). VAC 2: `Repository::fork` finds the merge-base with the configured base branch;
`history::check` also compares decisions with the fork; `invariant_history` compares each invariant's statement,
detail, oracle and marked test text with the fork and accepts a change only when a new decision names its ID.

## Verification

`just run clippy`, `just run rustfmt`, `just lint` and `just candidate check --only memory` pass. `just run test --
project_memory change_delivery shared::vcs` passes 88 tests, including the two-commit merge test and the invariant
test with and without a new decision.

## Blockers

None.

## Next action

Commit VAC 2, then VAC 3: AC2 decision markers only in files their decision links (D216).
