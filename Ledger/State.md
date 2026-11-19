# State

## Focus

P163 (B165), final VAC: pin the enabled revision and archive P163 with its Backlog card B165.

## Workspace

- Branch: feature/max-fallback
- Revision: 84afb65439c0b777cb64ab4500b103b5748038be

## Progress

- `just bootstrap HEAD` installed 84afb65 and wrote it to stable.txt.
- P163 moved to Archive/Plan with its Delivery and the AC1 verdict; B165 moved to Archive/Backlog; the D209 card
  and delivery links point to the archive.

## Verification

- `just check --only memory` passes after the rotation.
- The gate of 84afb65 passed with 564 tests.

## Blockers

None.

## Next action

Commit this VAC, then run `just feature-merge` and continue with P164 (B166).
