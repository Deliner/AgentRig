# State

## Focus

P191 (B198): the architecture rule reads and parses sources on several threads.

## Workspace

Branch: feature/parallel-architecture
Revision: 215ef4af24742df4c1d7fb41b793b147d8d96184

## Progress

VAC 1 `215ef4a` saved B198 and selected it as P191. VAC 2 makes `extract` in contract_check/analysis.rs read and
parse each source through `in_parallel` and collect the results by path, so both the crate sources and the rule's
scope are parsed on several threads with the same per-file result.

## Verification

Release candidate on AgentRig, `just candidate lint`, warm runs of the architecture measurement: before on `679083e`
832, 835, 829, 832 and 830 ms; after 269, 257, 255, 262, 263 and 273 ms. rustfmt, clippy and candidate lint are clean.

## Blockers

None.

## Next action

Commit VAC 2 through the gate, run the full test suite, then archive P191 and B198 in VAC 3 and run feature-merge.
