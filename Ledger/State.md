# State

## Focus

P149 (B150): remove `just scratch`; the pin and the lint baseline run without it (D197).

## Workspace

Branch: feature/no-scratch
Revision: 252ceabb15b8c87a726eb2556fae533ecb13bbb8

## Progress

VAC 1 removes the scratch command, recipe, hook branch, task area and merge cleanup from the runtime, rig and `init`
template; the hook passes `just lint-baseline` and the rig's catalog `bootstrap`; `bootstrap REVISION` installs that
revision and only then writes stable.txt. Docs, D102/D182 links and D197 updated.

## Verification

`just run test -- agent_hooks`: 26 passed, including `shell_guard_runs_required_steps_and_refuses_scratch`.
`just run test -- command_runner change_delivery::git::tests::cleanup shared::testing project_setup::initialization`:
61 passed, including `bootstrap_pins_a_revision_only_after_installing_it` and the rig comparison. `just lint` clean;
`just --list` names no scratch.

## Blockers

None.

## Next action

Commit VAC 1, then pin with `just bootstrap HEAD`, archive P149 and B150 and run `just feature-merge`.
