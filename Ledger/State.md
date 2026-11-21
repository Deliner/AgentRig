# State

## Focus

P172 delivers B178 entry-bypass. VAC 9 makes the structural_lint entry a table of contents: location, modes,
analysis, evaluation, lifting and baseline move into rule_evaluation, the output format into reporting, and
outside callers use the commands, baseline and lint file checks the entry reexports.

## Workspace

Branch: feature/entry-boundaries
Revision: 563d7d3d5c96862f0ad72bb09eb8ac5666101c84

## Progress

- rule_evaluation holds Location, Mode, analyze, evaluate, lifting, error_count and record_baseline.
- lint_files::location is renamed baseline_path; the entry reexports it with only_adds_rules and only_lowers_counts.
- structural_lint declares dependents 7 by measurement under AC6, six before by default.
- Decision links follow the moved owners.

## Verification

- `just run rustfmt` clean, `just run clippy` clean, `just test structural_lint::` 345 passed.
- Candidate lint with the entry rules: entry-use 192 to 177, entry-bypass 160 to 145, architecture clean.

## Blockers

None.

## Next action

Commit VAC 9, then route file discovery, Rooted, coverage modules and the rule identities of project_setup/lint.rs
through their entries.
