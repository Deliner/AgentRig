# State

## Focus

P175 delivers B182: each group of copies with the same behavior is replaced by one owner, behavior unchanged.
VAC 4 moves two test fixtures into shared testing: initialized, a temporary project after a successful init, replaces
eight copies, and commit_identity, the Test author, replaces nine.

## Workspace

Branch: feature/faster-commit
Revision: a88ef15d93de0bb1d8bb4941230ea3aa6bf4088f

## Progress

VAC 1 saved B182 as P175. VAC 2 added Repository::require and git::finish. VAC 3 added neutral_metrics values and
sole. Init calls with other arguments or a preceding git init keep their own code; shared/vcs/tests/fixture.rs sets
the identity through its own command and stays.

## Verification

just test shared::testing compiled all 614 tests without warnings and passed 4. The commit gate runs the affected
groups.

## Blockers

None.

## Next action

Commit VAC 4, then move the write that creates parent directories into shared testing.
