# State

## Focus

P174, B181: make the gate faster and the rig code simpler without changing behavior. VAC 3 reads many committed
blobs through one `git cat-file --batch` process instead of one Git process per file.

## Workspace

Branch: feature/faster-gate
Revision: 28011d3fe4354211fc72a84ba272bf5d227e8111

## Progress

VAC 2 batched invariant oracle discovery. VAC 3 adds `Repository::blobs` in shared/vcs/objects.rs, which asks one
process for each blob after reading the previous one, so no pipe can fill. The decision history check reads every
committed decision detail of a revision in one call, and revision export writes each file as it arrives, so its
memory peak stays one file. Both read objects from the tree they already listed, so contents are the same.

## Verification

memory-check on this repository took 7.8 s after VAC 2 and 3.2 s after VAC 3 with a warm build; about 460 Git
processes read decision details before. `just test shared::vcs` passed 8, `just test project_memory` 43 and
`just test change_delivery::gate` 14. `just lint`, `just run rustfmt` and `just run clippy` are clean.

## Blockers

None.

## Next action

Commit VAC 3, then measure the test groups and the gate stages again and remove the next largest avoidable cost.
