# State

## Focus

P168 (B179): tests live only in a `tests/` directory. Current VAC: phase 2 of D223, the test-code patterns become
`**/tests/**` alone and `test-placement` covers Python in the rig and in `init`.

## Workspace

Branch: feature/tests-directory
Revision: 8aa4ba9a7520b955810e389cc4e9666a44cc848c

## Progress

The user approved `lint.yaml` without `**/tests.rs` and with `.py` and `.pyi` for `test-placement`. `TEST_CODE` is
`**/tests/**`; the `init` override is gone. The Python example test moved to `Project/tests/test_sample.py`; the
fixtures, README and `anchor-at-crate` follow.

## Verification

`just lint` and `rustfmt` report nothing; `structural_lint::`, oracle, delivery and the I023 tests pass.

## Blockers

None.

## Next action

Run `just propose-pin HEAD` and ask the user for `! just approve`; commit the pin, archive P168 and B179, then run
`just feature-merge`.
