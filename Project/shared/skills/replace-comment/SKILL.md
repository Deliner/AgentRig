---
name: replace-comment
description: Remove a comment that is not an exact decision or invariant reference by moving its knowledge to its owner or deleting it, when comment-reference reports a finding.
---

# Keep knowledge where it is owned

Source comments may only reference a decision or an invariant. Everything else
a comment says either belongs to another owner or is not needed.

1. Read the comment and the code it describes. Establish what it tells a
   reader that the code does not.
2. Choose its owner, telling a choice from a property with norm-or-choice;
   one record may serve several comments:
   - Nothing beyond the code, or obsolete: delete it.
   - The code is unclear without it: rename or restructure the code until it
     states the same meaning, then delete it.
   - A contextual choice: reference the existing decision, or record a new one
     with edit-decisions when the choice is durable, as `DECISION: DNNN`.
   - A required, checkable property: reference the invariant whose test
     verifies it, or record one with edit-invariants, as `INVARIANT: INNN`.
   - A tool directive: fix what the tool reports, or configure the tool.
3. Write the reference alone on its line in the exact form; an explanation after
   the identifier is still a forbidden comment. A Python docstring follows the
   same steps.

Do not create placeholder records or cite an unrelated identifier to keep text.
Preserve behavior: if removing a directive changes execution and no other owner
can carry it, report the conflict instead of weakening the rule. Rerun the
reported lint check and the affected behavior checks. Apply
complexity-discipline.
