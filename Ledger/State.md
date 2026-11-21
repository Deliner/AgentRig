# State

## Focus

P186 (B193): the invariant history parses each current linked test once and skips a fork version equal to it.

## Workspace

Branch: feature/invariant-parses
Revision: 5d416d8493e491a5fa99a39f543317c7c7a6f533

## Progress

VAC 3 archives P186 and B193 with the history step and memory check timings and the acceptance results. VAC 1
`b609658` saved the card; VAC 2 `5d416d8` shares the source cache and skips a fork version equal to the current test.

## Verification

`just test` passes 617 tests, `just lint` reports nothing. History step median 186 → 138 ms, memory check median
502 → 447 ms on AgentRig.

## Blockers

None.

## Next action

Commit VAC 3, then run `just feature-merge`; the user pins the merged head with `! just approve`.
