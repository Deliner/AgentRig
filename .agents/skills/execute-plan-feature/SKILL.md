---
name: execute-plan-feature
description: Implement or resume the one active user-facing feature in Ledger/Plan.md. Use for product work governed by the Discipline Worker feature plan; do not use for unrelated repository administration.
---

# Execute the active feature

Read Ledger/Plan.md and the detail for its single active entry. Treat the feature, user capability, and acceptance sections as the required result; do not infer implementation requirements from their examples or wording.

Read applicable decisions and invariants, inspect Project and Git state, and choose the smallest implementation that satisfies the feature. The executing agent owns implementation choices. Record a decision only when multiple admissible approaches require a durable contextual choice, and record an invariant only for must-hold observable behavior with an executable pytest oracle.

Keep product implementation under Project. Change worker tooling or policy only when the active feature cannot be completed under the existing contract.

From `master`, start a retained feature branch with `just feature-start -- <name>` before editing. Prepare coherent commits there, run focused tests while iterating, then run `just check`. Mark the feature complete only when its observable acceptance behavior passes, commit that state, and integrate it with `just feature-merge`. If current `master` has advanced, let that command rebase and re-verify the feature before merging. Never delete the merged feature branch.

Activate another existing feature only when the completed feature no longer has required work; do not invent a next feature.
