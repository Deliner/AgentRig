# State

## Focus

P192 (B199): directories Git knows no files in are checked for ignore rules by one Git call per pass.

## Workspace

Branch: feature/batched-ignore
Revision: ba8c0e5687d9f920894b622f4bb0f5b3a2d95e3a

## Progress

VAC 1 saved B199 and selected it as P192. VAC 2 lets shared/vcs/directories.rs walk the known directories, then pass
all unknown directories of one pass to a single `git check-ignore --no-index --stdin -z` and walk the visible ones.

## Verification

Release candidate on AgentRig, `just candidate lint`, warm runs of the file and directory inventory: before on
`e9097d7` 98, 96, 95, 116 and 116 ms; after 39, 41, 40, 41 and 41 ms. `just test inventory` passes 9 tests;
Clippy and candidate lint are clean.

## Blockers

None.

## Next action

Commit VAC 2, then archive P192 and B199 with the timings and the acceptance results as VAC 3.
