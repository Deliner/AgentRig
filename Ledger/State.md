# State

## Focus

P148 (B149) AC1: a catalog write running mkdir, cp, mv, rm, touch or chmod names only the hook routes its operand paths
reach; Git and `just scratch` stay opaque (D196).

## Workspace

Branch: feature/write-hints
Revision: 504ee0a3e80ed7373a489737e37930633fa722d2

## Progress

- `shell_guard::written` reads the operands of a path program as root-relative paths; an option with `=` or `/` or an
  unresolvable operand leaves the command opaque.
- `skill_requirements::writes` names shell routes whose pattern's literal directory prefix overlaps an operand.
- `shell_event` uses the paths when present and the opaque list otherwise; D196 records the choice and narrows D047.

## Verification

- `just run test -- agent_hooks`: 25 passed, including `path_program_writes_name_only_the_routes_their_paths_reach`.
- `just lint`: passed.

## Blockers

None. P147 stays paused on the user's choice of the rig root repair.

## Next action

Commit this VAC, pin the gate-accepted head (D051), archive P148 and B149, then `just feature-merge` and start P149.
