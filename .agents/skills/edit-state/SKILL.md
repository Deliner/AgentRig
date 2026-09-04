---
name: edit-state
description: Update Ledger/State.md with a compact factual handoff for interruption recovery or a fresh session.
---

# Edit current state

- Read State, Plan, Git status/diff, and recent commits first. State is a replaceable snapshot; Git and current contracts override stale notes. It grants no new authorization.
- Keep these nonempty sections in order: Focus, Workspace, Progress, Verification, Blockers, Next action. Use explicit none or not run when applicable.
- Record the current authorized task/feature, observed branch and baseline revision, current VAC and uncommitted work, completed results, actual check outcomes, blockers, and the next concrete action. Link details instead of copying the Ledger or conversation.
- Distinguish planned, attempted, failed, and verified work. After a simulation/test failure, preserve the relevant command, observed result, and recovery step; do not claim success before it occurs.
- Refresh at meaningful VAC/task boundaries, blockers, handoffs, and before a known interruption or context reset. Include it in the relevant VAC; no extra commit per tool call. An abrupt crash may leave it stale.
- Preserve unrelated work. On branch handoff, reconcile State for the destination; after resume, verify recorded facts before acting. Never put secrets or full transcripts here.
