# State

## Focus

P190 (B197): lint reads and parses the sources selected by syntax rules on several threads.

## Workspace

Branch: feature/parallel-lint
Revision: 07ccd4b900d63dd475157e8c4a495bc42f27f5f3

## Progress

VAC 1 `07ccd4b` saved B197 and selected it as P190. VAC 2 parses the selected sources that are not yet analyzed
through `in_parallel` in file_measures.rs; evaluation.rs splits the syntax path into `syntax_rule_findings`, which
keeps the selection order of findings and of the first error.

## Verification

Release candidate on AgentRig, `just candidate lint`, warm runs of the rule evaluation: before on `3a92de1` 2526, 2521,
2510 and 2522 ms; after 1381, 1508, 1348, 1352, 1359 and 1359 ms. rustfmt, clippy and candidate lint are clean.

## Blockers

None.

## Next action

Commit VAC 2 through the gate, run the full test suite, then archive P190 and B197 in VAC 3 and run feature-merge.
