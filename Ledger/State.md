# State

## Focus

P166 (B173), VAC 3: the rig adopts D218. The approved settings set `git.install: bootstrap`, add the `propose-pin`
catalog command and recipe, make bootstrap.sh install without pinning and extend the rig difference list.

## Workspace

Branch: feature/pin-approval
Revision: 3637c88e801304146cb1e3bfa1f2557a3043d857

## Progress

- The user ran `just bootstrap HEAD`, which installed and pinned 3637c88, then `just approve`, which applied the
  proposals for git.yaml, commands.yaml, development.just, bootstrap.sh and the rig difference list and approved them
  with the pin.
- launchers.rs bootstrap test installs without pinning; AGENTS.md, CLAUDE.md and the distribution guide tell agents to
  run `just propose-pin HEAD` and ask for approval; D218 links the rig owners and D213 drops its bootstrap links.

## Verification

- `just config-check` passes; `just candidate check --only memory` passes.
- `just run test -- project_setup::initialization::tests::rig bootstrap_installs shell::`: 12 passed.
- Live: the hook refuses `just bootstrap HEAD` and names propose-protected-change.
- The first commit gate failed only `named_commands`: the guide named `just approve`; reworded, 2 passed.

## Blockers

None.

## Next action

Commit VAC 3, archive P166 and B173, then run `just propose-pin HEAD` and ask the user for `! just approve`.
