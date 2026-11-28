# State

## Focus

P201 (B140 AC1–AC2), plan-only VAC: record the results of the checks E1, A1 and A3 in Plan/201.md Delivery and
pause P201, because no tested rule change meets AC2.

## Workspace

Branch: feature/structure-reference-rank
Revision: b26c541b4627d02ec06adc8d57bcd1903474fad9
Only Ledger/Plan.md, Ledger/Plan/201.md and State are staged; the check edits and the probe in Project are
restored, and only the untracked v36 tree in .cache/b140/v36 remains.

## Progress

- AC1: E1 confirms the cause (without unshown v36 is first of 17, tree rho -0.06).
- A1 and A3 match their predictions: v36 drops to tenth with tree rho 0.683 and 0.682, so AC2 is not met.
- Delivery records the results table, the AC2 outcome, the blocker and the resume condition; the P201 row goes
  from active to paused.

## Verification

The commit gate checks this staged tree.

## Blockers

AC2 needs a user decision: another AC2 target, a tree outside the fitting, or a non-lexical measure.

## Next action

Commit the pause, switch to master, start the P202 (B137) branch and carry over the plan commits.
