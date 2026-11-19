# State

## Focus

P154 (B155 AC1–AC2), VAC2: the rig's commands use `access`, and the boolean `read_only` form is gone (D203).

## Workspace

Branch: feature/write-sandbox
Revision: cb7faef6ba44068968994110e2a6fb675a2c0919

## Progress

- VAC1 `cb7faef` pinned with `just bootstrap HEAD`; the installed runtime reads `access`.
- The rig's `read` is `read-only`, its `write` is `project` and the `git` of `write` is `unconfined`.
- Commands no longer accept `read_only` or a boolean `access`; the rig difference for `write` is removed.

## Verification

- `just config-check` passes with the pinned runtime.
- `just run test -- command_runner project_config project_setup agent_hooks`: 132 passed, including the rejected
  `access: true` and `read_only: true`.
- Live: `just write -- cp Ledger/State.md` to a home path failed with "Operation not permitted" and no file appeared;
  `just write -- touch` inside `.cache` worked; `just read -- git log` worked.

## Blockers

None.

## Next action

Commit VAC2 through `just write -- git commit`, which runs the gate from Git's hooks; then pin it with
`just bootstrap HEAD`, archive P154 and B155, commit and merge.
