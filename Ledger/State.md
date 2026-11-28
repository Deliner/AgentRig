# State

## Focus

P205 (B209) starts: one card per task in three directories, a queue key and card numbers. This VAC activates P205
and records the delivery order in its Delivery.

## Workspace

Branch: feature/RIG-209-one-card-per-task
Revision: 6cfea74

## Progress

- P205 is active in Ledger/Plan.md; Plan/205.md Delivery records the branch and the order: a runtime that accepts an
  optional key and its consented pin, the protected proposal for the user, then the required key, its pin and the
  migration of the rig memory.

## Verification

- just check --only memory runs before this commit.

## Blockers

None.

## Next action

Add the optional key to project.yaml and the optional init --key with its validation and tests (step 1).
