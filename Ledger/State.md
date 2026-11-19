# State

## Focus

P151 (B152) hook checks every tool call, VAC1 for AC3: path resolution takes the spelling the file system stores, so
`ledger/State.md` on a case-insensitive file system gets the routes of `Ledger/State.md` (D199).

## Workspace

Branch: feature/hook-every-tool
Revision: 52d728b2961159227a34e843ac07e94d3f57e57f

## Progress

`shared::paths::resolve` replaces each existing non-link component with the last name of its canonical path. P151 is
active. Remaining: AC1 classifies every tool and refuses unknown ones, AC2 makes a failed hook launch exit 2.

## Verification

Without the fix the new route test failed on macOS; with it `just run test` passed 536 tests. Candidate lint, Clippy
and the memory check pass.

## Blockers

None.

## Next action

Commit VAC1, then register the hook for every tool and classify tools in the dispatcher (AC1, AC2).
