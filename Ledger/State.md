# State

## Focus

P160 (B162): a lint error for a match classifying three or more enum variants into true and false that repeats a
classification written earlier in the project. This VAC pins the inert kind and enables it with the cross-file report.

## Workspace

- Branch: feature/repeated-classification
- Revision: 4baca49e68038094ae0eb452d2facde21b30d672

## Progress

- stable.txt pins 4baca49, the inert kind, installed by `just bootstrap HEAD`.
- The kind is in the Rust handler and the catalog; evaluation reports a measured classification only where it repeats
  an earlier one of the same rule, in path and line order, and appends the first location to the message.
- The skill `classify-in-the-enum` is added with its link, group entry and init entry; lint.yaml enables the rule.

## Verification

- `just run test -- repeated_classification`: 3 passed, including the lint test on four files.
- `just run test -- project_setup`: 57 passed.
- `just lint` with the rule enabled passes; the rig has no repeated classification.
- `just run rustfmt`, `just run clippy`, `just lint-config-check` and `just check --only memory` pass.

## Blockers

None.

## Next action

Commit this VAC, run `just bootstrap HEAD`, then archive P160 and B162 with the pin and run `just feature-merge`.
