# State

## Focus

P205 (B209, D242), R2a VAC D: `init` creates `Backlog/`, `Plan/` and `Archive/` in the memory directory and writes no
Plan or Backlog index; its AGENTS.md describes one card per task, the card the session start names and `results/`.

## Workspace

Branch: feature/RIG-209-one-card-per-task. Revision: 1ce96cb79631d3abd56dfc3149dcd1408fd51566 plus the VAC D diff.
The installed runtime is c60786d; the candidate is built from this branch.

## Progress

- `Catalog` keeps only Decisions and Invariants; the optional `Delivery` section that only Plan used is gone, so
  `sections` and `sections_of` take the required headings alone.
- `project_memory` exports `CARD_DIRECTORIES` and `PLAN`; `init` creates the three directories beside the runtime
  directory, and the session start falls back to `PLAN` instead of a second literal.
- D242 links `init`, its instructions and the layout test; D050 no longer links the shipped instructions.
- The shared `memory()` fixture writes no Plan index; `nested_cwd_and_symlinks` no longer removes an init Plan.md.

## Verification

- `just run rustfmt` and `just run lint` pass on the VAC D tree.
- `just run test -- project_setup project_memory agent_hooks::tests::memory agent_hooks::tests::tools
  change_delivery::git::tests::pin`: 128 passed, 0 failed.
- Not run: the full suite; the commit gate runs the mapped groups.

## Blockers

- Protected changes wait for the user's own `! just approve`: `key: RIG`, `--key RIG` in bootstrap.sh, the card routes
  in hooks.yaml, the lint excludes and the rig_differences entry, staged under `.cache/runtime/proposal/`.

## Next action

Commit VAC D, then VAC E: the edit-backlog, edit-plan and execute-plan-feature skills describe one card and its moves.
