# State

## Focus

P154 (B155 AC1–AC2), VAC1: `just write` programs change only the project, and a sandboxed process cannot start a
program outside the sandbox (D203).

## Workspace

Branch: feature/write-sandbox
Revision: 4e231ff6055fc9c7ea15e619c36843583534834a

## Progress

- `access` (`read-only`, `project`, `unconfined`) replaces `read_only` in commands and job records; a listed program
  may set its own `access`.
- A `project` run uses the platform sandbox writing also the project root; the macOS sandbox refuses `lsopen` and
  `appleevent-send`.
- Init's `write` is `project` and its `git` is `unconfined`, because Git's hooks run the gate, whose tools write
  caches outside the project.
- Until the pinned runtime reads `access`, the candidate also accepts the boolean `read_only`, and the rig's `write`
  without `access` is a listed rig difference.
- D203 recorded.

## Verification

- `just run test`: 550 passed before the last simplification; after it
  `just run test -- command_runner project_config project_setup agent_hooks`: 132 passed.
- Without `lsopen` the escape test fails; with `write` unconfined the outside test fails; with `git` confined the hook
  test fails.
- `just lint` and `just run clippy` pass.

## Blockers

None.

## Next action

Commit VAC1, run `just bootstrap HEAD`, switch the rig's commands to `access`, remove the boolean form and the rig
difference, pin again, probe live, archive P154 and B155, then merge.
