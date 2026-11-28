# State

## Focus

P205 (B209): pin c60786d mid-branch, the runtime that accepts an optional queue key, so that project.yaml and the
installer can name key RIG before the key becomes required.

## Workspace

Branch: feature/RIG-209-one-card-per-task
Revision: c60786d

## Progress

- just propose-pin HEAD pinned c60786d under the consent of the user and installed it; stable.txt names it.

## Verification

- The branch gate of propose-pin passed at c60786d before the install.

## Blockers

None.

## Next action

Commit stable.txt with State, then write the proposal of the protected files: key RIG in project.yaml, --key RIG in
bootstrap.sh, card routes in hooks.yaml, card exclusions in lint.yaml and the rig differences of the old archive.
