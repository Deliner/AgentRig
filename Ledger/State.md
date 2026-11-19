# State

## Focus

P157 (B158): pin the revision with D214 to D217 so the installed runtime enforces them on the rig (D051).

## Workspace

Branch: feature/gate-base-rules
Revision: f36bfba94bdc3d69183ab0a293b81d5457601b2d

## Progress

VAC 1 to VAC 4 committed (D214, D215, D216, D217). `just bootstrap HEAD` installed `f36bfba` and wrote it to
`tooling/distribution/stable.txt`.

## Verification

`just bootstrap HEAD` built and installed the revision and printed `agentrig 0.3.0`.

## Blockers

None.

## Next action

Commit the pin, then propose `checks.yaml`, `hooks.yaml` and the rig difference list for `! just approve`.
