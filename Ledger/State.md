# State

## Focus

P165 (B168), final VAC: pin the revision that enables `parallel-index` and archive P165 with its Backlog card B168.

## Workspace

- Branch: feature/parallel-index
- Revision: 0d9d4817ac70f11bc0d8d8a10125b59c2990fc7d

## Progress

- `just bootstrap HEAD` installed 0d9d481 and wrote it to stable.txt.
- P165 moved to Archive/Plan with its Delivery and the AC1 verdict; B168 moved to Archive/Backlog; the D211 card
  and delivery links point to the archive.

## Verification

- `just check --only memory` passes after the rotation.
- The gate of 0d9d481 passed the whole suite, 566 tests.

## Blockers

None for P165. P147, P156 and P157 stay paused until the user answers their open questions.

## Next action

Commit this VAC, then run `just feature-merge` and report the batch.
