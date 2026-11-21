# State

## Focus

P176 delivers B183: each Project code file that lint warns about by file-size is split by responsibility, or stays
with its reason, behavior unchanged. VAC 8 splits the initialization layout tests, 358 lines, by layout kind.

## Workspace

Branch: feature/cohesive-files
Revision: 4ac544fcf8491a0babbfa4cb10dd5b3ae751f17f

## Progress

VACs 1 to 7 split the silent-error detector, rust/mod.rs, rule evaluation, the contract rule tests, the Rust resolver
tests and the silent-error tests; template.rs stays because both partitions exceed a directory-size or dependents
budget. VAC 8 moves the Rust crate layout tests into rust_layout.rs and the consumer, initialize and flow fixtures
into tests/mod.rs; the D136 init test link follows its marker.

## Verification

just test initialization::tests passed the same 30 tests without build warnings; rustfmt on the staged files and
memory passed.

## Blockers

None.

## Next action

Commit VAC 8 through the gate, then list the remaining file-size warnings and split or justify each.
