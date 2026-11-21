# State

## Focus

P192 (B199): directories Git knows no files in are checked for ignore rules by one Git call per pass.

## Workspace

Branch: feature/batched-ignore
Revision: e9097d7937a55ec72fbcea05f8ab812a68664dd6

## Progress

VAC 1 saves B199 and selects it as P192 by the user's standing instruction. The batched ignore check in
shared/vcs/directories.rs is in the working tree, not staged.

## Verification

Release candidate on AgentRig, `just candidate lint`, warm runs of the file and directory inventory: before on
`e9097d7` 98, 96, 95, 116 and 116 ms; after 39, 41, 40, 41 and 41 ms. `just test inventory` passes 9 tests.

## Blockers

None.

## Next action

Commit VAC 1, then commit the batched ignore check as VAC 2.
