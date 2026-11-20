# State

## Focus

P171 (B176 entry-use) and P172 (B178 entry-bypass), one branch from master, autonomous under the user's consent
(D226). VAC 1: the two rule kinds with their detectors, skills, tests and decisions D227 and D228, before the pin.

## Workspace

Branch: feature/entry-boundaries
Revision: 0d20ef8687317efbcd51802b4e408029cf37c919

## Progress

- Python imports target the imported module; packages passed on the way are an initialization relation (D227).
- `architecture/entries.rs` measures own uses of an entry and uses past another directory's entry; contracts accept
  `group: true`. `contract_check/dependency_rules.rs` runs file-cycle, entry-use and entry-bypass from one list.
- Kinds `entry-use` and `entry-bypass` with skills `move-out-of-entry` and `go-through-entry`; `init` leaves them
  out until the rig configuration names them. Tests of the three dependency rules live in `tests/dependency_rules/`.

## Verification

- `just test structural_lint::` passed 344; `just test project_setup::` passed 58; `just lint` clean;
  `just check --only memory` passed.

## Blockers

None.

## Next action

Commit VAC 1, pin it under consent with `just propose-pin HEAD`, then measure both rules on the rig and repair.
