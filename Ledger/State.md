# State

## Focus

P164 (B166): `length-subtraction` also reports `.count_ones()` and `.count_zeros()` and a count passed alone
through a call. This VAC only makes P164 active.

## Workspace

- Branch: feature/bit-count-subtraction
- Revision: 1820f921a40479a7d2d1cc3858e4897bdec2321f

## Progress

- P163 merged into master; the batch continues with P164 in numeric order.

## Verification

- `just check --only memory` after the Plan edit.

## Blockers

None.

## Next action

Commit the activation, then extend the detector, its test, the skill text and a new decision in one VAC.
