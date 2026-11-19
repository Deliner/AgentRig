# State

## Focus

P160 (B162): a lint error for a match classifying three or more enum variants into true and false that repeats a
classification written earlier in the project. This VAC adds the inert kind with its detector, tests and D206.

## Workspace

- Branch: feature/repeated-classification
- Revision: 4c2e5d3af4dc3e708cc3590641946ace74a3a7cc

## Progress

- The `repeated-classification` kind exists in the control-flow family with its descriptor; the Rust analysis
  measures each qualifying match with its canonical classification as the symbol.
- It is not yet in the Rust handler, the catalog or lint.yaml, and evaluation does not yet compare across files.
- D206 records the choice, including the identity of a variant by the last enum segment and its name.

## Verification

- `just run test -- repeated_classification`: 2 passed.
- `just run rustfmt`, `just run clippy`, `just lint` and `just check --only memory` pass.

## Blockers

None.

## Next action

Commit this VAC, run `just bootstrap HEAD`, then enable the kind with the cross-file report in evaluation, a lint
test on two files, the skill `classify-in-the-enum` and the rule in lint.yaml, committing the pin with it.
