---
name: edit-state
description: Record compact factual recovery context after progress, failure or handoff.
---

# Preserve the information needed for correct continuation

Maintain a current recovery picture, not an accumulating history. Another
executor must be able to distinguish the authorized outcome, actual progress,
verified evidence and remaining action without reconstructing the conversation.

1. Reconcile the current instruction and contract with observed work and
   verification. Recorded assertions lose authority when live evidence differs.
2. Replace superseded claims rather than appending competing versions. Retain
   earlier context only when it changes how the remaining work must proceed.
3. Distinguish completed work, incomplete work, failure and uncertainty.
   Verification applies only to the content and scope actually checked.
4. Identify the next concrete action and any condition genuinely preventing it.

Do not turn a snapshot into authorization, another task database or a transcript.
Preserve unrelated work and omit secrets. Refresh at meaningful boundaries,
interruptions, blockers and handoffs; stop when the recovery picture is accurate.

## Repository binding

Use Focus, Workspace, Progress, Verification, Blockers and Next action in order.
Read current Plan and Git before editing. Record the task, VAC, observed branch
and revision, checks, blockers and next action. Branch: name and Revision: hash
must use the observed identity, not a moving branch reference. Read resume for
merge/rebase facts and reconcile after branch switches; a passing check for
another content state or HEAD is not current verification.
