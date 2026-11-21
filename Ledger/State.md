# State

## Focus

P176 delivers B183: each Project code file that lint warns about by file-size is split by responsibility, or stays
with its reason, behavior unchanged. VAC 2 splits the Rust silent-error detector into its three detectors.

## Workspace

Branch: feature/cohesive-files
Revision: 7230856368642017a2a9ca33339ec8f58a5131a2

## Progress

VAC 1 saved B183 and selected it as P176. VAC 2 replaces languages/rust/errors/silent_error.rs, 402 nonblank lines,
with collapsed_chain.rs for D097, D165 and D207, error_verdict.rs for the D097 Err arm and the Result<bool> and Ok
readers, process_success.rs for D126 and call_syntax.rs for the zero-argument method receiver both use. verdict_call
reuses ok_argument. The D097, D126, D165 and D207 links follow the new owners.

## Verification

just test errors:: passed 15 tests without build warnings; rustfmt on the staged files and the memory check passed.

## Blockers

None.

## Next action

Commit VAC 2 through the gate, then split the next oversized file and rerun lint with warnings.
