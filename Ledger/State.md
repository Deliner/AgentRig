# State

## Focus

P164 (B166), final VAC: pin the revision with the extended rule and archive P164 with its Backlog card B166.

## Workspace

- Branch: feature/bit-count-subtraction
- Revision: 14dc249250dfe642f5086b85fee3f4151b8a3246

## Progress

- `just bootstrap HEAD` installed 14dc249 and wrote it to stable.txt.
- P164 moved to Archive/Plan with its Delivery and the AC1 verdict; B166 moved to Archive/Backlog; the D210 card
  and delivery links point to the archive.

## Verification

- `just check --only memory` passes after the rotation.
- The gate of 14dc249 passed with 504 tests.

## Blockers

None.

## Next action

Commit this VAC, then run `just feature-merge` and continue with P165 (B168).
