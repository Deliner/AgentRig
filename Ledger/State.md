# State

## Focus

P168 (B179): tests live only in a `tests/` directory. Current VAC: phase 1 of D223, the runtime that reports
`tests.rs` files and Python `test_*.py` or `*_test.py` files outside `tests/` and accepts Python for `test-placement`.

## Workspace

Branch: feature/tests-directory
Revision: b112f44643e8b71f3f49006363f930b1066075e9

## Progress

D223 is recorded. Each language handler names its test files; analysis reports a misplaced one at line 1; a Rust
declaration is placed only by a `tests` component. The skill `relocate-tests` and testing-discipline follow D223.
`init` keeps the Rust extension for the kind until phase 2, so its output still matches the rig (I023).

## Verification

`just lint` reports no finding; `structural_lint::` tests pass (329) and the I023 test passes.

## Blockers

Phase 2 needs the user's approval of the pin at this commit, because the installed runtime rejects `.py` for the
kind.

## Next action

Run `just propose-pin HEAD` and ask the user for `! just approve`; then propose `lint.yaml` without `**/tests.rs` and
with Python extensions for `test-placement`, drop the `init` override and `**/tests.rs` from the test-code patterns.
