# State

## Focus

P185 (B192): the memory history reads the fork revision once and keeps one reader of committed memory.

## Workspace

Branch: feature/history-reads
Revision: a362a1ca005500ad800b0905401f72ecc1379170

## Progress

VAC 1 saves B192 and selects it as P185 by the user's standing instruction to improve the project autonomously. The
shared snapshot change and a temporary step probe are in the working tree, not staged.

## Verification

Before on `a362a1c`, release candidate on AgentRig, warm runs: history step 192, 189, 205, 202, 208 and 203 ms. With
the unstaged change: 177, 187, 186, 185, 155, 155, 156, 157 and 156 ms.

## Blockers

None.

## Next action

Commit VAC 1, then remove the probe, run the suite and lint and commit the shared snapshot.
