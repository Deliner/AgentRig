# State

## Focus

P157 (B158): apply D217 to the rig: native `lint` beside `candidate-lint`, and a protected rig difference list.

## Workspace

Branch: feature/gate-base-rules
Revision: e05e14d3ac4a4a97cf564c5423a36824c4816b47

## Progress

VAC 1 to VAC 4 and the pin committed. The user approved with `just approve` the proposals of `checks.yaml`,
`hooks.yaml` and `rig_differences.yaml`; the distribution guide now says repository lint runs twice.

## Verification

`just config-check` passes, `just check --only lint` passes with the installed runtime, and `just run test --
project_setup::initialization::tests::rig` passes the rig difference test.

## Blockers

None.

## Next action

Commit the approved files, archive P157 and B158, run `just bootstrap HEAD` and commit the pin, then `just
feature-merge`.
