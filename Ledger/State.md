# State

## Focus

P177 delivers B184: each Project code directory that lint warns about by directory-size is split into nested parts
by responsibility, or stays with its reason, behavior unchanged. VAC 3 regroups structural_lint/tests.

## Workspace

Branch: feature/cohesive-directories
Revision: afc2a316cdacf5d39c0988b31f9cfa019aa7a2b8

## Progress

VAC 1 and VAC 2 are committed. VAC 3 moves the lint-explain, lint-tree and lint-structure tests into
structural_lint/tests/inspection, the dependency rule tests under tests/architecture, whose fixtures they use, and the
file system and Git inventory tests into tests/architecture/inventory, sharing their fixture through filesystem.rs;
both directories drop to 10 entries, and the D225, D227 and D228 test links follow. The oracle-bound diagnostics and
languages tests and the impact-map-bound configuration and discovery tests keep their paths.

## Verification

just test structural_lint::tests passed 109 tests without build warnings, 614 in total; just lint, rustfmt on the
staged files and memory passed; just lint --warnings lists 11 Project code directories by directory-size instead of 13.

## Blockers

None.

## Next action

Commit VAC 3 through the gate, then regroup structural_lint/languages/tests, which has 15 entries.
