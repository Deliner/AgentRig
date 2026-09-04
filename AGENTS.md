# Repository instructions

## Resume before work

1. Read Ledger/Plan.md, the active feature detail, and relevant decision and invariant details.
2. Inspect Git status and recent commits. Preserve unrelated changes.
3. Apply the complexity-discipline skill for non-trivial design or implementation.
4. Use norm-or-choice before changing durable policy, decisions, invariants, or these instructions.

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
- Work on one active Plan feature. The executing agent chooses the implementation and records only genuine durable choices.

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
