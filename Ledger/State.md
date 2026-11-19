# State

## Focus

P164 (B166), code VAC: `length-subtraction` also reports `.count_ones()` and `.count_zeros()` and a count passed as
the only argument of a call; the skill `subtract-checked`, the descriptor and decision D210 follow.

## Workspace

- Branch: feature/bit-count-subtraction
- Revision: 84301609acb64edb21016d2b41f27f2b2eba1123

## Progress

- `control_flow/length_subtraction.rs` unwraps one call with a single argument and accepts the two bit counts.
- A new test in `tests/control_flow/mod.rs` covers the review's widened `count_ones()` and the skipped forms.

## Verification

- `just run test -- control_flow` passes, 20 tests; `just run rustfmt`, `just run clippy`, `just lint` and
  `just check --only memory` pass; the rig has no newly reported site.

## Blockers

None.

## Next action

Commit this VAC, run `just bootstrap HEAD`, then archive P164 and B166 with the pin and run `just feature-merge`.
