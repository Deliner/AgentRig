# State

## Focus

Correct the P205 (B209) resumption order in Plan/205.md: after P199-P204 merge, rebase onto master, pin the
rebased head, and only then approve the rig module with `key: RIG` and the bootstrap with `--key RIG`.

## Workspace

Branch: feature/card-queue
Revision: 389856b9b8e98a7fe6f5a66faf2dea6763ee16ef

## Progress

- A pin of master cannot read `key`, so approving the module before the rebase would stop commands and hooks once
  the rebase resolves tooling/distribution/stable.txt to the pin of master.
- Plan/205.md now orders the resumption as rebase, pin of the rebased head, approval, then the VAC that requires
  the key; the claim that the installed runtime accepts `key` is gone, since the rebase replaces that pin.

## Verification

- The commit gate checks the memory links and lint of this plan-only change.

## Blockers

- The rig `.agentrig/project.yaml` with `key: RIG` and `tooling/distribution/bootstrap.sh` passing `--key RIG` to
  init wait for the `! just approve` of the user after the rebase; the proposal waits in the session scratchpad as
  b209-proposal.

## Next action

Commit the correction, then on feature/decision-sections run the candidate memory check and pin P199 under consent.
