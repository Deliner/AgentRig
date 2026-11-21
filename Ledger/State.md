# State

## Focus

P189 (B196): the gate fingerprint of input files reads and hashes the files on several threads.

## Workspace

Branch: feature/parallel-fingerprint
Revision: bb1985e9b2c055873601187edd9c5ef310e54a7e

## Progress

VAC 1 saves B196 and selects it as P189 by the user's standing instruction, extended for 10 more hours. The parallel
fingerprint and the move of the parallel map into Project/shared are in the working tree, not staged.

## Verification

Before on `bb1985e`, release candidate on AgentRig, warm runs: attempt start 96, 104, 98, 96, 103 and 92 ms, finish
102, 102, 107, 101, 107 and 101 ms. With the unstaged change the start median is 68.5 ms and the finish median 78 ms.
`just test` passes 617 tests; `just lint`, clippy and rustfmt are clean.

## Blockers

None.

## Next action

Commit VAC 1, then commit the parallel fingerprint as VAC 2.
