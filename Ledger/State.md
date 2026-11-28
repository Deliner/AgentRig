# State

## Focus

P205 VAC E (AC6): the session start names the card whose number the current branch carries: `feature/RIG-7-name`
gives Ledger/Plan/RIG-7.md or Ledger/Plan/RIG-7/RIG-7.md, any other branch Ledger/Plan; resume reports the same card.

## Workspace

Branch: feature/card-queue
Revision: 7bc4955e1b79e9461dc11ac41a07db9aab95c302

## Progress

- cards.rs finds the card of a branch from the git prefix, the queue key and a number without a leading zero, and
  returns Plan/KEY-N.md or Plan/KEY-N/KEY-N.md when present, else Plan/; project_memory re-exports it as active_card.
- The session start reads the branch through Git and names that card, or the error in its place; resume no longer
  reads Plan.md and reports the path as card.
- The new hook test switches branches without Git hooks: the reference hook made each switch take about 14 seconds.
- The inventories and the project setup README name the active card in place of Plan.

## Verification

- `just run test agent_hooks::`: 47 passed; `just run test project_memory::`: 53 passed.
- `just run rustfmt`, `just lint` and `just candidate memory-check` pass; the memory check prints the known D112, D175
  and D231 warnings.

## Blockers

b209-proposal waits for `! just approve` after the P199 to P204 merge, the rebase of this branch and a pin of its head.

## Next action

Commit VAC E, then pause P205 on this branch with Delivery for c870287, 7bc4955 and VAC E, and switch the checkout
to feature/decision-sections for the approval of the P199 lint.yaml proposal.
