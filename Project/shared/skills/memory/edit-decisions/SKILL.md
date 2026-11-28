---
name: edit-decisions
description: Record contextual choices and maintain current application links.
---

# Preserve the reason a choice governs the current context

A decision records why one alternative was selected under particular conditions.
It must not silently become a universal requirement or change meaning as its
implementation moves. Apply norm-or-choice.

Establish the choice, its reasons and the alternatives considered from the
actual evidence. Preserve the identity and meaning of an existing committed
choice. When a different choice supersedes it, create an explicit successor
and make precedence clear rather than rewriting the old rationale.

Keep application links attached to the current owners. Moving or removing an
owner changes those links, not the historical decision. Verify that remaining
links and markers identify real applications; do not invent compatibility
behavior solely to preserve an old location.

## Repository binding

Keep stable IDs, statements and committed detail contents. A new detail
contains Decision, Reasons and Alternatives in this order, none empty. One
decision records one choice. Each reason is a checkable fact or a link to a
principle or decision. Each alternative names what it offered and why it was
not applied. A committed detail keeps its Context, Chosen, Rejected, Rationale
and Consequences; the memory check refuses that layout for a new decision and
does not check the content rules. Applies in identifies current owners; linked
supported source files carry DECISION: DNNN comments. Update actual coverage
and markers together. The index identifies successors; Git preserves previous
links.
