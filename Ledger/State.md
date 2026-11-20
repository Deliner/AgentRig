# State

## Focus

P168 (B179): tests live only in a `tests/` directory. Current VAC: the mid-branch pin of D223 phase 1, so the
installed runtime accepts Python for `test-placement`.

## Workspace

Branch: feature/tests-directory
Revision: 87b6f155a9cddc72c36e922c2cf5deb1ddbf99a3

## Progress

Phase 1 is committed: handlers name their test files and analysis reports a misplaced one at line 1. The user
approved the pin of 87b6f155a9cddc72c36e922c2cf5deb1ddbf99a3, which is installed and written to `stable.txt`.

## Verification

The pre-commit gate of 87b6f15 passed with 522 tests; `just approve` built and installed that revision.

## Blockers

None.

## Next action

Propose `lint.yaml` without `**/tests.rs` and with Python extensions for `test-placement`, ask for `! just approve`,
then drop the `init` override and `**/tests.rs` from the test-code patterns.
