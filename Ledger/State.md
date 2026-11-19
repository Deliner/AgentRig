# State

## Focus

P165 (B168), inert VAC: the `parallel-index` detector, its test, the `Reference::ParallelIndex` descriptor and D211,
not yet enabled in the rig, the catalog or init (D051).

## Workspace

- Branch: feature/parallel-index
- Revision: dafabea1d5b77aa598a3f4e822c0011d5f829065

## Progress

- `rust/naming.rs` reports an index from `.position(..)` on `.iter()`, `.iter_mut()` or `.into_iter()` that reads
  a collection with other text, directly, through `let`, `let .. else`, `if let` or a closure parameter.
- A test in `tests/reference/naming.rs` covers the review's touch zones and the skipped forms.

## Verification

- `just run test -- reference::naming` passes, 3 tests; `just run rustfmt`, `just run clippy`, `just lint` and
  `just check --only memory` pass. A reading of the rig's `.position(` sites finds no site.

## Blockers

None.

## Next action

Commit this VAC, run `just bootstrap HEAD`, then enable the kind with the skill `keep-pairs-together`.
