# State

## Focus

P190 (B197): lint reads and parses the sources selected by syntax rules on several threads.

## Workspace

Branch: feature/parallel-lint
Revision: 3a92de1066b36eda1305fcd08d511300df3080f1

## Progress

VAC 1 saves B197 and selects it as P190 by the user's standing instruction. The parallel source analysis in
structural_lint/rule_evaluation is in the working tree, not staged.

## Verification

Release candidate on AgentRig, `just candidate lint`, warm runs of the rule evaluation: before on `3a92de1` 2526, 2521,
2510 and 2522 ms; after 1381, 1508, 1348, 1352, 1359 and 1359 ms. rustfmt, clippy and candidate lint are clean.

## Blockers

None.

## Next action

Commit VAC 1, then commit the parallel source analysis as VAC 2.
