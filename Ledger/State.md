# State

## Focus

P171 and P172: entry-use and entry-bypass. VAC 14 splits project_setup into bundle and inspection so that
initialization uses siblings and project_setup offers init through its entry.

## Workspace

Branch: feature/entry-boundaries
Revision: 22e84257bf49a4bde5921edda8fca00d3788bcca

## Progress

- The generator files and the assembly moved into project_setup/bundle; Files, Links and skill_links into
  bundle/contents.rs.
- doctor and coverage moved into project_setup/inspection; project_setup/mod.rs only declares and offers.
- cli_dispatch, agent_hooks tests and project_setup tests call project_setup::init.
- Decision links follow the moved owners; five dependents budgets rose by one for the split, per the approved AC6.

## Verification

- Candidate entry lint: entry-bypass 2 to 0, entry-use 153 to 143, architecture clean.
- Clippy and rustfmt pass.

## Blockers

None.

## Next action

Commit VAC 14, then reduce entry-use clusters, starting with cli_dispatch Configured and the main.rs crate root.
