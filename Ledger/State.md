# State

## Focus

P157 (B158), VAC 4: the installed linter also checks a rig change (AC6), D217.

## Workspace

Branch: feature/gate-base-rules
Revision: 18923da5c64340412e68b5a04cc118d1b4142343

## Progress

VAC 1 (D214), VAC 2 (D215) and VAC 3 (D216) committed. VAC 4: a gate test shows a native `lint` check refusing an
oversized staged source that a `candidate-lint` command, standing for a weakened candidate, accepts; D217 records the
rig configuration: native `lint` plus `candidate-lint`, and the rig difference list joining the protected files.

## Verification

`just run clippy`, `just run rustfmt`, `just lint` and `just candidate check --only memory` pass. `just run test --
change_delivery::gate::tests::outcomes` passes 5 tests, including the new one.

## Blockers

None.

## Next action

Commit VAC 4, run `just bootstrap HEAD` and commit the pin, then propose `checks.yaml`, `hooks.yaml` and the rig
difference list for `! just approve`.
