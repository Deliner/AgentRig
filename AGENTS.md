# Repository instructions

## Resume before work

1. Read Ledger/State.md, Ledger/Plan.md, the active feature detail if present, and relevant decision and invariant details. Follow explicit superseding decisions; historical records do not override their successors.
2. Inspect Git status and recent commits. Preserve unrelated changes.
3. Apply the complexity-discipline skill for non-trivial design or implementation.
4. Use norm-or-choice before changing durable policy, decisions, invariants, or these instructions.

## Current state and editing guidance

- Treat State as a compact recovery snapshot, not a second Plan or VAC history. Compare it with current Git state and contracts before following its next action; interrupted work may have advanced beyond the last snapshot.
- Keep State's Focus, Workspace, Progress, Verification, Blockers, and Next action sections factual. Record the task, observed branch/revision, current VAC and pending work, actual check results, blocker, and concrete next step. Distinguish attempted work from verified results.
- Update State at meaningful VAC/task boundaries, blockers, handoffs, and before a known interruption or context reset, as part of the relevant VAC. Reconcile it when switching branches; avoid extra state commits after every tool call.
- Before editing Plan or its details, apply edit-plan; for Decisions, edit-decisions; for Invariants, edit-invariants; for State, edit-state. All four skills live under .agents/skills. Read only the matching skills.
- The shared Rust hook in tooling/worker routes file-specific guidance and existing complexity/command checks. Ledger guidance adds context without blocking edits. For opaque shell writes, apply matching skills yourself; the hook cannot determine arbitrary script targets.

## Commands

- Invoke shell operations only through a recipe shown by just list.
- Use just read for read-only inspection and just write for an explicitly unrestricted operation.
- Give every justfile recipe an immediately preceding one-line What/Why comment.

## Durable knowledge

- A decision records context, chosen and rejected alternatives, rationale, and consequences.
- An invariant states observable behavior and links an exact marked pytest test.
- Add the exact marker # DECISION: DNNN to every linked Python or shell implementation.
- Put # INVARIANT: INNN immediately before the linked test definition.
- Plan entries describe user-facing functionality and capability, not implementation.
- Work on at most one active Plan feature. The executing agent chooses the implementation and records only genuine durable choices. No active feature is a valid idle state; start only work authorized by the user or an authorized manager.

## Evolving the delivery plan

- Keep feature IDs stable and use Plan row order for delivery priority. Record actual prerequisites in `Depends on`; active and complete features require completed prerequisites.
- Add future features when grounded in current requirements, observed constraints, or user/authorized manager instructions. Describe their product results and source, then continue the current VAC unless priority or acceptance explicitly changes.
- Treat implementation difficulties within the current contract as VAC work. Split a feature only for independently meaningful outcomes or necessary prerequisites, not merely because it needs many edits.
- If a prerequisite blocks delivery, pause the current feature, record the blocker, retained branch, and resumption condition in its Delivery section, and put the prerequisite before it. Finish or discard only the current uncommitted VAC before switching; retain verified commits and unrelated work.
- Carry a plan-only handoff commit to the new prerequisite branch from master, so the prerequisite can be delivered without merging unfinished product code. On resume, reconcile the retained branch with current master and the latest Plan before another VAC. Follow the execution skill for the concrete sequence.
- Complete a feature only after its acceptance passes; record checks and results in Delivery. Required acceptance work cannot be deferred to a follow-up without an explicitly authorized contract change. Do not invent a next feature to keep the Plan active.

## Verified atomic changes

- Deliver each feature through verified atomic changes (VACs): cohesive changes that can be checked and reverted independently. One VAC may span several files and tool calls; it need not deliver the whole feature.
- Before a VAC, identify its intended result and a sufficient falsifiable check. Choose the next VAC from current evidence; do not preregister an implementation plan for the whole feature.
- Edit, run focused checks, and correct failures freely within the VAC. A failed check does not require a failed commit or special permission before correction.
- Stage only the coherent VAC, inspect the staged diff, and commit after its focused verification passes. Describe the result and verification in the commit message. Do not include unrelated pre-existing work.
- The pre-commit hook runs the full staged-tree gate. Do not routinely run `just check` and `just check-staged` immediately before the same commit; use them for diagnosis or an explicit verification need.
- Start the next VAC after the current one is committed or its own uncommitted changes are deliberately discarded. Preserve completed VACs as commits; no separate VAC ledger or edit checkpoint is required.

## Branches and integration

- Never commit directly on `master`. Start work from `master` with `just feature-start -- <name>`.
- Commit feature work only on its retained `feature/<name>` branch.
- Finish with `just feature-merge`; it rebases a divergent feature onto current `master`, verifies it, merges without flattening its commits, and keeps the feature branch.
- Do not delete a feature branch after it has been merged.
- Never bypass .githooks/pre-commit.
- Treat source-size warnings as a prompt to reassess ownership; use a refactoring skill only when structural decomposition is justified.

## Native runtime and configurable lint

- Registered hooks and Git guards use tooling/worker/run. The launcher verifies a source fingerprint before using its cached locked release build; do not bypass it with a stale binary.
- Structural lint is configured in tooling/worker/lint.toml and runs on staged commits and before integration. Use just lint for focused feedback and just lint-rules for supported target/language capabilities.
- Each diagnostic names a repair skill. Consider the warning skill for soft findings; apply the error skill to correct blocking findings. External gate failures also name their configured skill.
- Apply configure-linter before changing selectors, thresholds, rule kinds, or skill mappings. Do not weaken thresholds merely to pass. New compiled rules need declared capabilities, behavior tests, and repair guidance.
- Rust source changes participate in rustfmt, Clippy, and native behavioral tests. Use // DECISION: DNNN for Rust traceability; existing mandatory Python/shell markers remain unchanged.
