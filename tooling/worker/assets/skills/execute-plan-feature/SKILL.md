---
name: execute-plan-feature
description: Deliver or resume an authorized product feature through verified atomic changes and adapt its delivery plan when blockers or new requirements arise. Use for product work governed by the configured Plan; explicit worker-administration requests remain independently authorized.
---

# Execute feature delivery

Run `just resume` to observe Git operations, State revision and the latest check evidence; reconcile those facts before choosing the next step. An unfinished check is not proof that a process is still running. Read agentrig.yaml for paths and Git base/prefix, then the configured State and Plan, the active feature detail if present, applicable decisions and invariants, and Git state. Reconcile State's recorded task, current VAC, verification, blockers, and next action against the actual branch, diff, and recent commits; a crash can leave it stale. Follow explicit superseding decisions. Treat Feature, User capability, and Acceptance as the result contract; the executing agent chooses architecture and implementation. A feature may be a substantial MVP capability or an observable improvement such as optimization.

Plan contains only outcomes explicitly selected for execution now in the current session or resumed task. At most one feature is active. If none is active, use an existing current delivery instruction to select its ready pending entry or resume its paused one; otherwise stop. Backlog is an unordered collection of future ideas and hypotheses, never an automatic execution queue. A request to save or plan something for later belongs there. Context reset or interruption alone does not cancel current authorized work.

Keep product implementation in the configured source locations. Change worker policy or tooling only when explicitly requested or when delivery cannot satisfy the existing contract without it. Record decisions only for genuine durable contextual choices, and invariants only for must-hold behavior with an executable configured test oracle.

## Start and implement

From a clean configured base branch, use `just feature-start <name>`. For existing work, resume its retained branch instead of creating a replacement. Record useful branch/resume context in the feature's Delivery section; Git contains the completed VACs.

Before each verified atomic change (VAC), state in the current context: "After this change, X becomes possible; verify it through Y." Choose an observable result and a falsifiable check. A VAC is one cohesive, independently checkable and revertible change, potentially spanning several files, tool calls, and corrections. It need not deliver a whole feature. Choose the next step from current evidence without prescribing all future implementation steps.

Edit, run focused checks, and correct freely within the VAC. Add or update tests when needed to verify changed behavior; a no-op or a prose edit does not require invented tests. Failed checks permit immediate correction. Inspect and stage only the coherent change, then describe the intended result, observed verification and any remaining limits in the commit message. A passing check is evidence only for the behavior it exercises. The pre-commit hook runs the configured staged-tree gate; explicit affected-path groups may select test targets. Do not routinely repeat that gate manually before committing; use `just check --only CHECK_ID` for a focused retry, adding `--staged` when checking the index. Selective test evidence does not prove the full suite passed. Integration runs the full gate; never replace either required gate with an agent-selected subset.

Finish or deliberately discard only the current uncommitted VAC before beginning the next. Preserve unrelated work. Completed VACs are commits; there is no separate VAC registry, per-edit lock, or requirement to provoke a failed commit before fixing code.

Use edit-state to refresh the compact State snapshot at meaningful VAC boundaries, blockers, handoffs, and before known interruptions/context resets. Include updates in the relevant VAC. Record only observed verification results and distinguish integration still to be performed from integration already observed in Git. On branch handoff or resume, reconcile State for the destination rather than carrying stale completion claims. Use edit-plan, edit-decisions, and edit-invariants before changing their respective memory files or details.

## Adapt the plan

Classify an obstacle against current acceptance:

- A technical difficulty within the current contract stays inside the feature. Change the implementation or next VAC.
- A necessary independent prerequisite pauses the current feature and goes before it in delivery order. Add its ID to the paused feature's Depends on.
- An improvement outside current acceptance goes to Backlog, without delivery priority. Put it in Plan only when explicitly selected for execution now.
- New instructions may select additional outcomes for the current session. Record that scope and actual prerequisites, keeping one active feature. Requests to remember future work go to Backlog. Do not interrupt the current VAC unless the instruction changes current scope or makes that VAC obsolete.

New Plan entries require selection for current execution or a prerequisite necessary for that accepted scope. Keep stable IDs, outcome-based details and the reason/source in Delivery. An observed opportunity alone belongs in Backlog. Do not turn every implementation step into a feature or split solely because work is large.

When work reveals an improvement or hypothesis outside current acceptance, use edit-backlog to retain it and its context, then resume the task. Recording an idea does not expand delivery scope. Work needed for current acceptance must still be completed or reported as a blocker.

If later work is required for current acceptance, the current feature remains incomplete. Perform that work inside it or explicitly revise the result contract under the user's authorization. Do not silently shrink acceptance to manufacture completion.

## Pause and hand off

The steps below apply to a temporary prerequisite handoff within selected current work. If execution is deferred to a later task, use edit-backlog to preserve requirements, progress, blocker, retained branch and resumption condition, then remove the Plan row/detail and reconcile dependencies. Preserve unfinished work; interruption alone is not deferral.

1. Inspect the current diff. Finish and verify the current VAC if it remains useful, or discard only its own uncommitted changes. Do not reset the branch's verified commits, delete its branch, or discard unrelated work.
2. In a separate plan-only VAC, set the current feature to paused, record the concrete blocker, retained branch and condition for resumption in Delivery, and add the necessary prerequisite before it. Activate the prerequisite if its own dependencies are complete and delivery is authorized; otherwise leave zero active features. Commit only the relevant Plan index and detail changes and note that commit's hash.
3. With a clean working tree, return to the configured base branch using `just run write -- git switch <base>`, create the prerequisite branch with `just feature-start <name>`, and carry over the plan-only commit with `just run write -- git cherry-pick <plan-commit>`. Inspect the carried diff; it must contain no unfinished product code. Resolve any Plan conflict against current configured base branch without dropping newer entries or completion states.
4. Deliver the prerequisite there. If the blocker instead requires external input and no prerequisite can proceed, stop with the retained branch and resume condition recorded; do not create speculative features.

The prerequisite branch carries the updated Plan to the configured base branch through its normal integration. Never merge the paused feature merely to publish a plan change. Existing unrelated work that prevents a clean switch must be preserved separately or left in its current checkout.

## Resume

After the prerequisite is accepted and integrated, switch to the paused feature's retained branch from a clean working tree. Reconcile it with current configured base branch using `just run write -- git rebase --rebase-merges <base>`. Resolve conflicts using the latest delivered Plan as the baseline: preserve newly completed prerequisites and later instructions rather than restoring stale active states.

Reassess retained VACs against the new prerequisite and current acceptance. Dependencies must be complete and no other feature active before setting this feature active. Update its Delivery context and verify affected behavior before committing the resumption VAC. Reuse valid work and revise obsolete work; do not assume old verification proves the rebased result.

## Complete and integrate

Verify observable acceptance and record actual checks and results in Delivery, the relevant VAC and State. Apply edit-plan to automatically rotate completed rows and cards into Archive in the final VAC, preserving IDs, acceptance, evidence and live links. Current work may reference completed archived prerequisites. Unmet acceptance remains required work. An empty Plan is valid; start another outcome only when explicitly selected for current execution.

Commit the completion state and run `just feature-merge`. It rebases divergent work onto current configured base branch, verifies the integrated candidate, and merges without flattening feature commits. Keep the feature branch. After conflicts or rebase, reconcile Plan state and re-verify the affected result.

Plan changes, VACs and product completion have different boundaries: saving an idea does not authorize execution, a passing VAC does not prove full acceptance, and delivery reaches the configured base branch only after integration succeeds. The final VAC can archive completed Plan entries; failed integration remains current work recorded in State, not a reason to claim completion.
