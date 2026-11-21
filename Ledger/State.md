# State

## Focus

P176 delivers B183: each Project code file that lint warns about by file-size is split by responsibility, or stays
with its reason, behavior unchanged. VAC 4 splits rule_evaluation/evaluation.rs.

## Workspace

Branch: feature/cohesive-files
Revision: 7240befd5edc119b303f606447ebc96cc362f681

## Progress

VACs 1 to 3 selected the card, split the silent-error detector and emptied rust/mod.rs of two detector groups.
VAC 4 moves the structural file measures, D036, D072 and D195, into rule_evaluation/file_measures.rs and the finding
construction with the mixed-junction finding, D156, into findings.rs; evaluation.rs keeps the orchestration and falls
from 380 to 236 lines. D036, D072 and D195 link file_measures.rs; D156 also links findings.rs.

## Verification

just test structural_lint passed 346 tests without build warnings; rustfmt on the staged files and memory passed.

## Blockers

None.

## Next action

Commit VAC 4 through the gate, then split the next oversized file and rerun lint with warnings.
