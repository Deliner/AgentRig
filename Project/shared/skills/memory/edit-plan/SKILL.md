---
name: edit-plan
description: Maintain outcomes and acceptance explicitly selected for execution now; send future ideas to Backlog.
---

# Keep current execution tied to the agreed outcome

A plan selects work to execute now; recording an idea does not select it.
Preserve the agreed result, acceptance and authorization across planning,
delivery, interruption and archival.

## Select and continue

Read the cards in Plan and Backlog and State; reconcile them with Git and the
current instruction. Every new product outcome requires an agreed Backlog card
through edit-backlog. Selection requires agreed behavior and acceptance,
resolved behavior-changing questions and the user's selection of execution and
order. Do not take another Backlog card automatically. Explicit maintenance is
independently authorized.

Selecting a card moves it unchanged from Backlog to Plan instead of
paraphrasing it into a second specification. Preserve criterion IDs, wording
and the executor's ownership of implementation choices. One instruction to
execute a selected set authorizes all ready cards without repeated permission.
Interruption or context reset does not cancel unfinished authorized work.

An After line must justify execution order, not introduce unrelated outcomes.
Use execute-plan-feature for delivery and prerequisites; a new product outcome
cannot bypass agreement by being called a prerequisite. Work necessary for
current acceptance cannot be deferred merely to claim completion.

## Defer or complete without losing meaning

Only the user defers execution. Then record progress, blocker, branch and
resumption condition in the card's results/, move the card back to Backlog and
reconcile the After lines that name it. A blocker alone keeps the card in Plan
with that record through execute-plan-feature; interruption alone is not
deferral.

Archive a completed card automatically in the final VAC without another
confirmation: record its acceptance verification in results/ and move the card
unchanged from Plan to Archive. An archived card keeps its number and is not an
execution queue.

## Repository binding

Read paths.memory and key in .agentrig/project.yaml. A card is KEY-N.md, or the
directory KEY-N/ holding KEY-N.md and only assets/, results/ and scenarios.md;
its first line is `# KEY-N — title` and its sections are Context, Proposal and
Expected benefit. It lies in exactly one of Backlog/, Plan/ and Archive/ in the
memory directory and moves between them as the same file or directory with the
same number; a card that gains materials becomes the directory. No index lists
the cards, and an archive kept from the earlier Plan and Backlog indexes stays
as it is.

Plan runs in ascending order of numbers. An `After: KEY-M, KEY-K` line under
the heading runs a card after the named cards, which exist and form no cycle.
The active card is the one whose number the current branch carries as the
configured prefix followed by KEY-N-name; the session start names it. An empty
Plan is valid.

Never reuse or renumber a card. Update relative links into a moved card and
current application links; run `just check --only memory` after a move. Use
edit-backlog for out-of-scope ideas, save only agreed drafts and resume
authorized work. Use execute-plan-feature for verification and integration.
