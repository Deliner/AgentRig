---
name: edit-plan
description: Maintain outcomes and acceptance explicitly selected for execution now; send future ideas to Backlog.
---

# Keep current execution tied to the agreed outcome

A plan selects work to execute now; recording an idea does not select it.
Preserve the agreed result, acceptance and authorization across planning,
delivery, interruption and archival.

## Select and continue

Read configured Plan, Backlog and State; reconcile them with Git and the current
instruction. Every new product outcome requires an agreed Backlog source through
edit-backlog. Promotion requires agreed behavior and acceptance, resolved
behavior-changing questions and the user's selection of execution and order.
Do not take another Backlog item automatically. Explicit maintenance is
independently authorized.

Reference the same accepted contract instead of paraphrasing it into a second
specification. Preserve criterion IDs, wording and the executor's ownership of
implementation choices. One instruction to execute a selected set authorizes all
ready items without repeated permission. Interruption or context reset does not
cancel unfinished authorized work.

Dependencies must justify execution order, not introduce unrelated outcomes.
Use execute-plan-feature for delivery and prerequisites; a new product outcome
cannot bypass agreement by being called a prerequisite. Work necessary for
current acceptance cannot be deferred merely to claim completion.

## Defer or complete without losing meaning

If execution is explicitly deferred, preserve requirements, progress, blocker,
branch and resumption condition in Backlog before removing the current entry
and reconciling dependencies. Interruption alone is not deferral.

Archive completed outcomes automatically in the final VAC without another
confirmation. Preserve identity, acceptance and actual delivery evidence before
removing the live entry. A conflicting archived identity must not be overwritten.
Completed history is not an execution queue.

## Repository binding

Keep ID, Status, Depends on, Feature and User capability columns and stable PNNN
IDs. Details contain Feature, User capability and Acceptance; paused/complete
entries require Delivery with blocker/resumption context or observed verification.
Feature and Acceptance reference the agreed Backlog card; retain its agreed Git
revision when committed. User capability retains the agreed benefit.

Row order sequences selected work. Pending is selected, active is being delivered,
paused is temporarily blocked and complete has verified acceptance ready for
archival. At most one item is active; an empty Plan is valid. Dependencies may
reference current or archived outcomes, remain acyclic, and must be complete
before dependent items become active/complete.

Use the configured memory directory's Archive/Plan.md and Archive/Plan/NNN.md,
with the same columns, IDs, acceptance and Delivery. Create them when needed.
Do not restore archived entries merely to reference them or reuse their IDs;
allocate identities across Plan, Archive and Backlog references.

Update relative links, provenance, current application links and architecture
inventories; run memory-check after rotation. Use edit-backlog for out-of-scope
ideas, save only agreed drafts and resume authorized work. Use
execute-plan-feature for verification and integration.
