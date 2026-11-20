---
name: replace-comment
description: Remove a comment that is not an exact decision or invariant reference by moving its knowledge to its owner or deleting it, when comment-reference reports a finding.
---

# Keep knowledge where it is owned

Source comments may only reference a decision or an invariant, as the exact
`DECISION: DNNN` or `INVARIANT: INNN`, or be an exception
`DECISION: DNNN RULE` that lifts a finding of the rule RULE on the line
directly below it. Everything else a comment says either belongs to another
owner or is not needed.

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
   - Why a lint finding on the next line is intended: apply the repair skill
     the finding names, which fixes the code or records the exception
     `DECISION: DNNN RULE`; a bare reference lifts no finding.
3. Write the reference or exception alone on its line in the exact form; an
   explanation after the identifier or the rule ID is still a forbidden
   comment. A Python docstring follows the same steps.

## An exception that lifts nothing

The finding "the exception DNNN RULE lifts no RULE finding on the line below"
names an exception whose rule no longer fires on the line directly below it,
because that line was fixed, changed or moved.

1. Read the line below the exception, the decision it names and its
   `[exception RULE](FILE)` link under Applies in.
2. When the justified finding is gone, delete the exception and, with
   edit-decisions, its `[exception RULE](FILE)` link in the decision unless
   the file keeps another exception of that decision and rule.
3. When the justified line moved, move the exception alone directly above that
   line, and update its link when the file changed. A moved exception is new:
   the user approves it again through propose-protected-change.
4. Rerun the reported lint check.

Do not create placeholder records or cite an unrelated identifier to keep text.
Preserve behavior: if removing a directive changes execution and no other owner
can carry it, report the conflict instead of weakening the rule. Rerun the
reported lint check and the affected behavior checks. Apply
complexity-discipline.
