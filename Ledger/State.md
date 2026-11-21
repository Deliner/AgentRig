# State

## Focus

P186 (B193): the invariant history parses each current linked test once and skips a fork version equal to it.

## Workspace

Branch: feature/invariant-parses
Revision: b6096580af9ff382832f9309d257d05f12a0d453

## Progress

VAC 2: the memory check creates its source cache before the history step and passes it to the invariant history,
which prepares the current linked tests in parallel and reads them from that cache. A fork version with the same
file name and bytes as the current test is not parsed. A new oracle test keeps an invariant whose test file changed
only outside its function.

## Verification

`just test` passes 617 tests: the 616 of master `f3545d3` and the new one. Dropping the parse fallback fails the new
test. `just lint` reports nothing. History step median 186 → 138 ms.

## Blockers

None.

## Next action

Commit VAC 2, then archive P186 and B193 with the timings and acceptance results and run `just feature-merge`.
