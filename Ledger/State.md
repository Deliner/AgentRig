# State

## Focus

P189 (B196): the gate fingerprint of input files reads and hashes the files on several threads.

## Workspace

Branch: feature/parallel-fingerprint
Revision: 88e35dd15f98fafc2239535c3f70cd15584fe294

## Progress

VAC 1 `88e35dd` saved B196 and selected it as P189. VAC 2 moves the ordered parallel map from project_memory/source.rs
into Project/shared/parallel_map and makes input_fingerprint read and hash each file on several threads, feeding the
digest in path order. The change is staged.

## Verification

Before on `bb1985e`, release candidate on AgentRig, warm runs: attempt start 96, 104, 98, 96, 103 and 92 ms, finish
102, 102, 107, 101, 107 and 101 ms. After: start median 68.5 ms, finish median 78 ms. `just test` passes 617 tests;
`just lint`, clippy and rustfmt are clean.

## Blockers

None.

## Next action

Commit VAC 2, then archive P189 and B196 with the timings in VAC 3 and run feature-merge.
