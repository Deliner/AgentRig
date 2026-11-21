# State

## Focus

P176 delivers B183: each Project code file that lint warns about by file-size is split by responsibility, or stays
with its reason, behavior unchanged. VAC 9 moves the control-flow threshold tests out of languages/tests/mod.rs.

## Workspace

Branch: feature/cohesive-files
Revision: ee8d8db8f05c1cfb5bb69994de58fe229124697a

## Progress

VACs 1 to 8 split the silent-error detector, rust/mod.rs, rule evaluation, the contract rule, Rust resolver,
silent-error and initialization layout tests; template.rs, shell_guard.rs, project_memory/mod.rs, skills.rs and
reference.rs stay with recorded reasons. VAC 9 moves the nesting, branch budget and boolean-expression tests into
control_flow/thresholds.rs, which reads through the symbol and line readers tests/mod.rs keeps for declarations and
embedded text; languages/tests/mod.rs drops from 469 to 248 lines and the D061, D063 and D120 links follow.

## Verification

just test structural_lint::languages::tests passed the same 174 tests without build warnings; just lint, rustfmt on
the staged files and memory passed.

## Blockers

None.

## Next action

Commit VAC 9 through the gate, then examine markers.rs, project_setup/tests/defaults.rs and delivery.rs.
