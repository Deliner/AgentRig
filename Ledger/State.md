# State

## Focus

P195 / B202: the linter leaves signatures that foreign code sets. VAC1 delivers Rust (AC1–AC4, AC8, AC9 Rust text):
`parameter-count`, `boolean-parameter` and `tuple-as-record` skip extern-block functions and methods of impls of
traits whose path leads to a declared external crate or a prelude trait; decision D229.

## Workspace

Branch: feature/foreign-signatures
Revision: 94b8c2aaaf4dbff1dbf93d42a64ab898161fa35d

## Progress

- `languages::analyze` takes a `Context` of pairs and an injected `Project` view; rule evaluation builds it from the
  directory-architecture settings (`rule_evaluation/foreign_code.rs`).
- `rust/exemptions/foreign_trait.rs` decides extern members and foreign trait paths through the file's imports.
- Language tests pass `DETACHED`; new tests in `tests/signature/foreign_signature.rs` and `rule_evaluation/tests`.
- Descriptors name the unmeasured signatures; D229 detail and index row written.
- VAC2 (Python `typing.override`, AC5–AC7, AC9 Python text) not started.

## Verification

- `just run test -- structural_lint::languages`: 178 passed; signature tests 41 passed after the split.
- `just run test -- structural_lint::rule_evaluation`: 9 passed.
- `just lint`, `just run rustfmt`, `just run clippy` and `just check --only memory`: pass.

## Blockers

None.

## Next action

Commit VAC1, then deliver VAC2 for Python.
