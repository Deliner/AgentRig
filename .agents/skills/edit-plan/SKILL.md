---
name: edit-plan
description: Edit Ledger/Plan.md or its feature details while preserving outcome contracts and valid delivery order.
---

# Edit the plan

- Read the current Plan and affected feature details. Describe product outcomes, user capability, and observable acceptance; leave architecture and VAC boundaries to the executing agent.
- Keep IDs stable. Use pending, active, paused, or complete, at most one active feature, and existing acyclic Depends on IDs. Active/complete features require complete prerequisites.
- Add work only from current requirements, observed constraints, or user/authorized manager instructions. Record the source in Delivery. Pause for independent blockers; do not disguise unfinished acceptance as a follow-up.
- Keep Feature, User capability, Acceptance, and optional Delivery sections in order. Paused entries require blocker, retained branch, and resume condition; completed entries require actual acceptance evidence in Delivery.
- Reconcile related rows/details together. Use the existing execution skill for branch handoff; update State when the next action changes. The staged gate validates the Plan schema.
