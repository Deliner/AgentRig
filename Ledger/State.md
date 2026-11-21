# State

## Focus

P172 delivers B178 entry-bypass. VAC 10 routes the remaining outside uses of structural_lint through its entry:
file discovery, Rooted, crate module paths for coverage and the rule kinds of project_setup/lint.rs; the lint
parts are declared privately so the compiler holds the boundary.

## Workspace

Branch: feature/entry-boundaries
Revision: ed5922c1ab0f2a88fff5b5999940b8831b28d7a4

## Progress

- file_discovery keeps Inventory in inventory.rs; collect is renamed maintained_files and reexported by the entry.
- capabilities keeps RULE_KINDS and kind parsing in kinds.rs; example is renamed example_rule.
- architecture/mod.rs offers crate_modules from file texts, so coverage reads the files and the lint only parses.
- structural_lint declares dependents 9 by measurement under AC6, 7 before.
- Decision links of D016, D017, D018, D133, D134 and D144 follow the moved owners.

## Verification

- `just run rustfmt` clean, `just run clippy` clean, `just test` 613 passed.
- Candidate lint with the entry rules: entry-use 177 to 152, entry-bypass 145 to 97, architecture clean.

## Blockers

None.

## Next action

Commit VAC 10, then route cross-feature uses of command_runner, project_memory, change_delivery, project_config
and project_setup through their entries.
