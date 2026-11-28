# State

## Focus

P205 (B209), plan-only VAC: resume P205 before P199-P204 merge for the parts that need neither the approval of
the user nor a new decision. The card check for entries named by the queue key (AC2-AC5) beside the old indexes,
consent over Plan/ cards (AC7) and the active card path at session start (AC6) follow as code VACs.

## Workspace

Branch: feature/card-queue
Revision: e3bee65617b021d6f2aeadebb98decebb0f794bd

## Progress

- Ledger/Plan.md: P205 paused -> active; no other item is active and P205 has no dependency.
- Ledger/Plan/205.md Delivery: the superseded resumption bullet is replaced by the resumption now and the order after
  the P199-P204 merge. The old index model stays until the last code VAC: removing it changes the oracles of I008
  and I009, D215 then needs a new decision, and the Decisions.md index takes a new row only after P199 AC5.
- Chosen: a paused card keeps its record in the State of its retained branch, a delivered card in Git history; the
  card itself holds neither (B210 AC1).

## Verification

- Plan-only change; the commit gate runs the installed memory check on it.

## Blockers

- Protected files of AC1 (b209-proposal/ in the session scratchpad) wait for the `! just approve` of the user after
  the P199-P204 merge, the rebase and a pin of the rebased head.

## Next action

Commit this plan-only VAC, then the VAC of the card check for entries named by the queue key.
