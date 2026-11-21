# State

## Focus

P172 delivers B178 entry-bypass. VAC 12 routes the uses of project_memory and project_config through their
entries: check_evidence, format and protection are declared privately and the entries offer their API.

## Workspace

Branch: feature/entry-boundaries
Revision: b23228ac9463e0cf9c4782bc5508db0e935263a6

## Progress

- project_memory offers Attempt, Input, native_index, report_checks, observed_checks, Catalog, table and the State
  template, reset and notes check.
- project_config offers Protection and protected_configuration; settings_schema is private.
- gate, git, cli_dispatch, agent_hooks and project_setup use these only through the entries.

## Verification

- `just run rustfmt` clean, `just run clippy` clean.
- Candidate lint with the entry rules: entry-bypass 64 to 38, entry-use 153, architecture clean.

## Blockers

None.

## Next action

Commit VAC 12, then route the uses of change_delivery gate and git through the delivery entry.
