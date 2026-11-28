# State

## Focus

P201 (B140 AC1–AC2), plan-only VAC: activate P201 and record in Plan/201.md Delivery the scores before any rule
change, the AC1 cause found so far and the predictions of the checks, written before they run.

## Workspace

Branch: feature/structure-reference-rank
Revision: a102ff92d51999c4715c8b40597ee04df29c75bb
Only Ledger/Plan.md, Ledger/Plan/201.md and State are staged. The worktree keeps an unstaged probe in
Project/structural_lint/reporting/structure/mod.rs (per-check sums) that never goes into a commit, and the
untracked v36 tree in .cache/b140/v36.

## Progress

- 17 trees on one base (P129 original, 15 P129 agent trees, v36 built from the original by the P128 map) are
  scored with one candidate revision; tree rho with Sonnet 0.677, agreement on 48 directories 0.583.
- Only the unshown check gives v36 more points than the v3 and v5 trees; the directories and the mechanism
  (agents repeat parent description words in child names) are recorded.

## Verification

The commit gate checks this staged tree.

## Blockers

None for this VAC.

## Next action

Run the checks E1, A1 and A3 recorded in Delivery as worktree-only edits on all 17 trees, compare the ranks
and agreement with the predictions, then record the results and the AC2 outcome.
