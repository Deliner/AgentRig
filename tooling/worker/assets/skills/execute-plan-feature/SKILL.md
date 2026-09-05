---
name: execute-plan-feature
description: Deliver or resume an authorized product feature through verified atomic changes and adapt its delivery plan when blockers or new requirements arise. Use for product work governed by the configured Plan; explicit worker-administration requests remain independently authorized.
---

# Execute feature delivery

Read worker.toml for paths and Git base/prefix, then the configured State and Plan, the active feature detail if present, applicable decisions and invariants, and Git state. Reconcile State's recorded task, current VAC, verification, blockers, and next action against the actual branch, diff, and recent commits; a crash can leave it stale. Follow explicit superseding decisions. Treat Feature, User capability, and Acceptance as the result contract; the executing agent chooses architecture and implementation. A feature may be a substantial MVP capability or an observable improvement such as optimization.

At most one feature is active. If none is active, use an already authorized delivery instruction to select a ready pending feature or resume a paused one; otherwise report the current state and stop. Row order expresses intended priority, not independent authorization. Do not invent more work merely to keep the Plan active.

Keep product implementation in the configured source locations. Change worker policy or tooling only when explicitly requested or when delivery cannot satisfy the existing contract without it. Record decisions only for genuine durable contextual choices, and invariants only for must-hold behavior with an executable configured test oracle.

## Start and implement

From a clean the configured base branch, use `just feature-start -- <name>`. For existing work, resume its retained branch instead of creating a replacement. Record useful branch/resume context in the feature's Delivery section; Git contains the completed VACs.

Before each verified atomic change (VAC), identify its intended result and sufficient falsifiable verification. A VAC is one cohesive, independently checkable and revertible change, potentially spanning several files, tool calls, and corrections. It need not deliver a whole feature. Choose the next step from current evidence without prescribing all future implementation steps.

Edit, run focused checks, and correct freely within the VAC. Add or update tests when needed to verify changed behavior; a no-op or a prose edit does not require invented tests. Failed checks permit immediate correction. Inspect and stage only the coherent change, then commit with the result and verification described in the message. The pre-commit hook runs the complete staged-tree gate. Do not routinely run the same full gate manually before committing; use `just check` or `just check --staged` for diagnosis or an explicit verification need.

Finish or deliberately discard only the current uncommitted VAC before beginning the next. Preserve unrelated work. Completed VACs are commits; there is no separate VAC registry, per-edit lock, or requirement to provoke a failed commit before fixing code.

Use edit-state to refresh the compact State snapshot at meaningful VAC boundaries, blockers, handoffs, and before known interruptions/context resets. Include updates in the relevant VAC. Record only observed verification results and distinguish integration still to be performed from integration already observed in Git. On branch handoff or resume, reconcile State for the destination rather than carrying stale completion claims. Use edit-plan, edit-decisions, and edit-invariants before changing their respective memory files or details.

## Adapt the plan

Classify an obstacle against current acceptance:

- A technical difficulty within the current contract stays inside the feature. Change the implementation or next VAC.
- A necessary independent prerequisite pauses the current feature and goes before it in delivery order. Add its ID to the paused feature's Depends on.
- A justified follow-up goes after the current feature while current delivery continues. Add a dependency on the current feature only if its outcome is actually required.
- New instructions from the user or an authorized manager may add several pending features during work. Record their source and requested outcomes, keeping one active feature. Do not interrupt the current VAC unless the instruction changes priority or makes that VAC obsolete.

New entries require a current requirement, observed constraint, or authorized instruction. Keep stable IDs, outcome-based details, and the reason/source in Delivery. Do not turn every implementation step into a feature or split solely because work is large.

If later work is required for current acceptance, the current feature remains incomplete. Perform that work inside it or explicitly revise the result contract under the user's authorization. Do not silently shrink acceptance to manufacture completion.

## Pause and hand off

1. Inspect the current diff. Finish and verify the current VAC if it remains useful, or discard only its own uncommitted changes. Do not reset the branch's verified commits, delete its branch, or discard unrelated work.
2. In a separate plan-only VAC, set the current feature to paused, record the concrete blocker, retained branch and condition for resumption in Delivery, and add the necessary prerequisite before it. Activate the prerequisite if its own dependencies are complete and delivery is authorized; otherwise leave zero active features. Commit only the relevant Plan index and detail changes and note that commit's hash.
3. With a clean working tree, return to the configured base branch using `just run write -- git switch <base>`, create the prerequisite branch with `just feature-start -- <name>`, and carry over the plan-only commit with `just run write -- git cherry-pick <plan-commit>`. Inspect the carried diff; it must contain no unfinished product code. Resolve any Plan conflict against current the configured base branch without dropping newer entries or completion states.
4. Deliver the prerequisite there. If the blocker instead requires external input and no prerequisite can proceed, stop with the retained branch and resume condition recorded; do not create speculative features.

The prerequisite branch carries the updated Plan to the configured base branch through its normal integration. Never merge the paused feature merely to publish a plan change. Existing unrelated work that prevents a clean switch must be preserved separately or left in its current checkout.

## Resume

After the prerequisite is accepted and integrated, switch to the paused feature's retained branch from a clean working tree. Reconcile it with current the configured base branch using `just run write -- git rebase --rebase-merges <base>`. Resolve conflicts using the latest delivered Plan as the baseline: preserve newly completed prerequisites and later instructions rather than restoring stale active states.

Reassess retained VACs against the new prerequisite and current acceptance. Dependencies must be complete and no other feature active before setting this feature active. Update its Delivery context and verify affected behavior before committing the resumption VAC. Reuse valid work and revise obsolete work; do not assume old verification proves the rebased result.

## Complete and integrate

Verify the feature's observable acceptance, then record the actual checks and results in Delivery and mark it complete. Unmet acceptance remains required work. Leaving zero active features is valid; activate another existing feature only under an authorized delivery instruction.

Commit the completion state and run `just feature-merge`. It rebases divergent work onto current the configured base branch, verifies the integrated candidate, and merges without flattening feature commits. Keep the feature branch. After conflicts or rebase, reconcile Plan state and re-verify the affected result.

Plan changes, VACs, and product completion have different boundaries: adding future work does not complete it, a passing VAC does not prove full feature acceptance, and a complete feature is delivered to the configured base branch only after integration succeeds.
