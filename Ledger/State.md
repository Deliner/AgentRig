# State

## Focus

P159 (B161), VAC 1: the inert `indexed-array-literal` kind with its detector, tests and D205, before the mid-branch
pin.

## Workspace

- Branch: feature/array-index-literal
- Revision: 6c1af207c60685185f6a01c321fcfeab9d71c187

## Progress

- Detector `std_forms/indexed_array.rs` reports an array literal listing elements with a call, indexed by a value
  other than an integer literal or `..`, test code included; a reference above lifts it.
- Descriptor `INDEXEDARRAYLITERAL` with skill name `branch-on-the-index`; D205 records the predicate.
- The kind stays out of the handler registry, the catalog, init and lint.yaml until the pin.

## Verification

- `just run test -- std_forms::indexed_array`: 2 passed, the review's example among the findings.
- `just lint`, `just run rustfmt` and `just run clippy` pass.

## Blockers

None.

## Next action

Commit this VAC, run `just bootstrap HEAD`, then enable the kind in the registry, the catalog, init and lint.yaml
with the skill branch-on-the-index and commit it with the pin.
