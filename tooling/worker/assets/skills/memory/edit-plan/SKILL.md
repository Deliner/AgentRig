---
name: edit-plan
description: Maintain outcomes and acceptance explicitly selected for execution now; send future ideas to Backlog.
---

# Keep Plan limited to current execution

Read the configured Plan, Backlog and State and reconcile them with Git and the current instruction. Plan holds only work explicitly selected for execution now in the current session or resumed task. A request to save or plan something for later goes to edit-backlog; it does not select implementation. Do not automatically take the next Backlog card.

Keep stable PNNN IDs, Status, Depends on, Feature and User capability columns. Details contain Feature, User capability and Acceptance; paused/complete entries require Delivery with blocker/resumption context or observed verification. Preserve acceptance and agent ownership of implementation choices.

Use row order only to sequence the selected current work. Pending means selected for this session, active means being delivered, paused means temporarily blocked within that work, and complete means verified acceptance ready for automatic archival. At most one entry is active; an empty Plan is valid. An interruption or context reset does not discard an unfinished authorized task.

Dependencies may refer to current Plan or completed Archive rows and remain acyclic; active/complete entries require completed prerequisites. Do not restore archived rows into Plan just to reference them. If a new prerequisite is needed now, use execute-plan-feature for the scoped handoff. If work is deferred, preserve its requirements, blocker, branch and resume condition in Backlog before removing its Plan row/detail and reconciling remaining dependencies.

Automatically rotate completed outcomes in the final VAC without asking for another confirmation. Use the configured memory directory's Archive/Plan.md index and Archive/Plan/NNN.md cards, keeping the same Plan columns, original PNNN IDs, acceptance and Delivery evidence. Create the archive when first needed. Preserve each card and its index row there before removing its live Plan entry; a different existing record with the same ID is a conflict, never an overwrite.

Update relative links for the new card location, Backlog provenance, current application links and architecture inventories. Current Plan dependencies can refer to completed archived IDs. Run memory-check after rotation. Archive is readable history, not an execution queue; it retains completed results rather than discarding them. Allocate new IDs across Plan, Archive and existing Backlog references, never reusing an old identity.

Use edit-backlog for observations, improvements and hypotheses outside current acceptance, then resume the task. Required current work cannot be deferred merely to claim completion. Use execute-plan-feature for verification and integration.
