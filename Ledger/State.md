# State

## Focus

P177 delivers B184: each Project code directory that lint warns about by directory-size is split into nested parts
by responsibility, or stays with its reason, behavior unchanged. VAC 2 regroups structural_lint/tests/architecture.

## Workspace

Branch: feature/cohesive-directories
Revision: e211718c99f2a2bed1c3e1a76c8d8c44ada0e1c7

## Progress

VAC 1 saved and selected B184 as P177. VAC 2 moves the Rust-only attribute, binding, crate, macro, resource and
wiring tests into structural_lint/tests/architecture/rust_resolution, which imports the parent fixtures by crate
path; the parent drops from 15 to 10 entries and the D136 test links follow.

## Verification

just test structural_lint::tests::architecture passed the same 47 tests without build warnings, 614 in total;
just lint, rustfmt on the staged files and memory passed.

## Blockers

None.

## Next action

Commit VAC 2 through the gate, then regroup structural_lint/languages/tests, the next directory with 15 entries.
