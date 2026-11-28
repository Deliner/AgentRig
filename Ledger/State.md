# State

## Focus

P205 VAC D (AC7): `just consent` records the key-named cards in Plan/ and refuses an empty Plan/; consent holds while
a recorded card stays in Plan/, so a card moved to Archive/ or a revoke returns propose-pin to a proposal.

## Workspace

Branch: feature/card-queue
Revision: c870287c7567717b309c3280b36263de2de4a6f3

## Progress

- cards.rs lists the cards named by the queue key in Plan/ (file or directory); project_memory re-exports it as
  planned_cards, and the table reader is no longer part of its API.
- consent.rs records and checks those cards instead of pending or active Plan.md rows; its messages name Plan/.
- The pin tests configure key RIG, write Ledger/Plan/RIG-1.md and move it to Archive/ for the delivered case.
- The hook refusal of `just consent`, the distribution README and the inventories describe consent by Plan/ cards.
- Plan/205.md lists the decision on consent by Plan/ cards, the successor of D226, for the end-game VAC.

## Verification

- `just run test change_delivery::git::tests::pin::`: 5 passed.
- `just run test project_memory::tests::`: 47 passed; `just run test agent_hooks::tests::protected::`: 6 passed.
- `just run rustfmt`, `just lint` and `just candidate memory-check` pass (the known D112, D175, D231 warnings).

## Blockers

b209-proposal waits for `! just approve` after the P199 to P204 merge, the rebase and the pin of the rebased head.

## Next action

Commit VAC D, then VAC E (AC6): the session start names the active card from the branch instead of Plan.md.
