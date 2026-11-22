# State

## Focus

Maintenance speed-up without behavior change: building the stageable index skips an `update-index` whose path
list is empty, since it leaves the temporary index unchanged; admitting one path always had one such call.

## Workspace

Branch: feature/empty-blob-request
Revision: 1cf44eb3a4c65b290789267eb5cf974f6b344554

## Progress

Project/shared/vcs/git.rs filters out empty path lists before running `update-index`.

## Verification

`just run test -- shared::vcs change_delivery::git project_setup`: 103 passed.

## Blockers

None.

## Next action

Commit through the hook, propose the pin and run feature-merge.
