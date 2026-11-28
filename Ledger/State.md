# State

## Focus

P205 (B209, D242), R2a VAC E: the edit-plan, edit-backlog and execute-plan-feature skills describe one card per task,
its moves between `Backlog/`, `Plan/` and `Archive/`, the order by number and `After:`, the active card of the branch
and the delivery record in `results/` (AC9).

## Workspace

Branch: feature/RIG-209-one-card-per-task. Revision: 74038ca95f62dce111f2cc88dfb42e101ee80ce5 plus the VAC E diff.
The installed runtime is c60786d; the candidate is built from this branch.

## Progress

- edit-plan: rewritten selection, deferral, archival and repository binding for cards named `KEY-N`; the archive kept
  from the earlier indexes stays as it is.
- edit-backlog: a card is saved in `Backlog/` with the next number of the three directories; selection moves it
  unchanged and deferral moves it back with its record in `results/`.
- execute-plan-feature: the active card comes from the branch, `feature-start KEY-N-name`, `After:` instead of
  Depends on, and `results/` instead of the Delivery section.
- D242 links the three skills; D050, which D242 supersedes, no longer links them.

## Verification

- `just run lint` passes on the VAC E tree.
- The skills are prose; the commit gate runs the mapped test groups.

## Blockers

- Protected changes wait for the user's own `! just approve`: `key: RIG`, `--key RIG` in bootstrap.sh, the card routes
  in hooks.yaml, the lint excludes and the rig_differences entry, staged under `.cache/runtime/proposal/`.

## Next action

Commit VAC E. Then, without approval, prepare the rig migration (AC8) text and report; with approval, the approval
commit, R2b (required key), `just propose-pin HEAD` and the migration.
