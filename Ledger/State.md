# State

## Focus

Maintenance speed-up authorized by the user's autonomous goal, with business logic unchanged. VAC: test fixtures
resolve git through git's exec path ahead of the inherited PATH, so test processes and the workers they start
skip the macOS xcrun shim on every git spawn.

## Workspace

Branch: feature/empty-blob-request
Revision: 747b797c508055aaf49fc53583a0c8a3833438c5
Changed: Project/shared/testing/tests/mod.rs (search_path and hermetic place the exec-path directory first).

## Progress

The empty blob request skip and the empty update-index skip are committed on this branch. This VAC changes only
the test helpers; product code is untouched.

## Verification

Three alternating full-suite runs: about 55 s wall and 111 s user with the exec path first, against about 67 s wall
and 130 s user with the shim; every test passed in all six runs. change_delivery::git::tests::guards passes.

## Blockers

None.

## Next action

Commit through the gate, run just propose-pin HEAD, then just feature-merge and compaction.
