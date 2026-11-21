# State

## Focus

P176 delivers B183: each Project code file that lint warns about by file-size is split by responsibility, or stays
with its reason, behavior unchanged. VAC 5 splits the contract rule tests, 499 lines, by behavior.

## Workspace

Branch: feature/cohesive-files
Revision: ccd43d56334e58da30428a0e0c31ff59694d7062

## Progress

VACs 1 to 4 split the silent-error detector, rust/mod.rs and rule evaluation. bundle/template.rs stays: a sibling
file lifts bundle above 10 entries and a nested template directory lifts the dependents of project_config and
shared/vcs to 14 and of shared/calling_agent to 8 above their budgets, which the gate refused; that attempt is undone.
VAC 5 moves the contract rule tests into contracts, features, cycles, zones and coupling files with the shared
fixtures in tests/mod.rs; the links of D070, D078, D079, D083 and D104 follow their markers.

## Verification

just test contract_rules passed the same 17 tests without build warnings; rustfmt on the staged files and memory
passed.

## Blockers

None.

## Next action

Commit VAC 5 through the gate, then split resolution/rust/tests/mod.rs by behavior.
