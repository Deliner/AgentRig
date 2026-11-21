# State

## Focus

P191 (B198): the architecture rule reads and parses sources on several threads.

## Workspace

Branch: feature/parallel-architecture
Revision: 679083e7eadb80d9bb86e58045962cd2349cb42e

## Progress

VAC 1 saves B198 and selects it as P191 by the user's standing instruction. The parallel extraction in
structural_lint/architecture/contract_check is in the working tree, not staged.

## Verification

Release candidate on AgentRig, `just candidate lint`, warm runs of the architecture measurement: before on `679083e`
832, 835, 829, 832 and 830 ms; after 269, 257, 255, 262, 263 and 273 ms. rustfmt, clippy and candidate lint are clean.

## Blockers

None.

## Next action

Commit VAC 1, then commit the parallel extraction as VAC 2.
