# State

## Focus

P176 delivers B183: each Project code file that lint warns about by file-size is split by responsibility, or stays
with its reason, behavior unchanged. VAC 7 splits the silent-error measurement tests, 317 lines, along the detectors.

## Workspace

Branch: feature/cohesive-files
Revision: 2f7f3221cf30d5c807c4ba7fa3785cf396fd1516

## Progress

VACs 1 to 6 split the silent-error detector, rust/mod.rs, rule evaluation, the contract rule tests and the Rust
resolver tests; template.rs stays because both partitions exceed a directory-size or dependents budget. VAC 7 moves
the silent-error tests into except_pass, collapsed_chain, error_verdict and process_success files with the shared
finding reader in tests/errors/mod.rs; the test links of D069, D097, D126, D165 and D207 follow their markers.

## Verification

just test languages::tests::errors passed the same 15 tests without build warnings; rustfmt on the staged files and
memory passed.

## Blockers

None.

## Next action

Commit VAC 7 through the gate, then split initialization/tests/layout.rs into general and Rust crate layout tests.
