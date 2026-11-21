# State

## Focus

P176 delivers B183: each Project code file that lint warns about by file-size is split by responsibility, or stays
with its reason, behavior unchanged. VAC 10 moves the eager-fallback and disabled-RAII tests out of markers.rs.

## Workspace

Branch: feature/cohesive-files
Revision: 40d33b94bd0b8e8c8d429882aff176d2aad19593

## Progress

VACs 1 to 9 split the silent-error detector, rust/mod.rs, rule evaluation, the contract rule, Rust resolver,
silent-error and initialization layout tests and moved the control-flow threshold tests; template.rs, shell_guard.rs,
project_memory/mod.rs, skills.rs and reference.rs stay with recorded reasons. VAC 10 moves the eager-fallback tests
into std_forms/eager_fallback.rs and the disabled-RAII test into ownership/disabled_raii.rs beside the detectors they
mirror; markers.rs drops from 378 to 274 lines and the D089, D092, D094 and D168 test links follow.

## Verification

just test structural_lint::languages::tests passed the same 174 tests without build warnings; just lint, rustfmt on
the staged files and memory passed.

## Blockers

None.

## Next action

Commit VAC 10 through the gate, then examine project_setup/tests/defaults.rs and delivery.rs.
