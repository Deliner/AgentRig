# Repository instructions

## Resume before work

1. Read ledger/STATE.md and relevant decision details.
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
- Keep ledger/STATE.md limited to current facts and the next action.

## Edit and verify

- Prepare one coherent edit. The checkpoint blocks a second edit until the changed contents are committed or reverted.
- Run just check while iterating and just check-staged before committing.
- Never bypass .githooks/pre-commit.
- Treat source-size warnings as a prompt to reassess ownership; use a refactoring skill only when structural decomposition is justified.
