# State

## Focus

P205 (B209, one card per task), stage R2a, VAC B: the session start and resume name the card whose number the
branch carries.

## Workspace

- Branch: feature/RIG-209-one-card-per-task
- Revision: da03ef9, with VAC B staged for its commit.
- Installed runtime: c60786d; `just run test` and `just run lint` build the candidate from the branch.
- Protected proposals waiting for `! just approve` lie in `.cache/runtime/proposal/`: `key: RIG` in project.yaml,
  `--key RIG` in bootstrap.sh, the card routes in hooks.yaml, the card excludes in lint.yaml and an entry in
  rig_differences.yaml.

## Progress

- VAC A is committed (da03ef9): the memory check reads cards instead of the Plan and Backlog indexes.
- VAC B: `project_memory::active_card` finds the card `KEY-N.md` or `KEY-N/` in Backlog, Plan or Archive whose
  number the branch `{prefix}KEY-N-name` carries. The session start names that card or else the Plan directory; a
  failed lookup names the Plan directory and writes its cause to stderr.
- resume no longer reads Plan.md and reports `card`: the path of that card or null.
- D242 links the session start, resume and their tests.

## Verification

- `just run test -- resume_names_the_card the_session_start_names_the_card`: 2 passed.
- `just run test -- agent_hooks::tests project_memory::tests`: 91 passed.
- `just run rustfmt`: clean; `just run lint -- --warnings`: warnings only, none in the changed files.
- `just check --only memory` (installed runtime): PASS.

## Blockers

The rig's Ledger keeps the old layout until the user's `! just approve` admits the protected proposals; the pin and
the migration then land in one commit.

## Next action

Commit VAC B, then VAC C: `just consent` records the cards in Plan/ and holds while one of them stays there.
