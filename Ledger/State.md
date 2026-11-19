# State

## Focus

P162 (B164): the inert Rust kind `outer-flag-filter` reports `.filter(..)` on a receiver that contains a call when its
closure takes only `_` and its body calls nothing. This VAC adds the kind, its detector, tests and decision D208.

## Workspace

- Branch: feature/outer-flag-filter
- Revision: 1882aef6aa9787c9577676c7ca6d986bb3226b1d

## Progress

- Detector `std_forms/flag_filter.rs`; the call search moved from `indexed_array.rs` into `std_forms/mod.rs` to serve
  both detectors.
- Descriptor `OUTERFLAGFILTER` with skill name `check-the-flag-first`; the kind is not yet in the registry, the
  catalog, init or `lint.yaml`.

## Verification

- `just run test -- std_forms`: 12 passed, two new tests on the review's forms and their boundaries.
- `just lint` with the candidate, `just run rustfmt`, `just run clippy`, `just check --only memory` pass.

## Blockers

None.

## Next action

Commit this VAC, run `just bootstrap HEAD`, then enable the kind with its skill and rewrite the rig's own sites.
