# State

## Focus

Maintenance at the user's request of 2026-11-22: AGENTS.md and CLAUDE.md open with a Purpose section that says what
AgentRig is and for what, adapted to the current rig, plus the criterion of a rig mechanism; not a Plan card. VAC 1:
the Purpose section in both files within the 70-line limit, decision D234 for the criterion and the user's request in
Ledger/Requests.md.

## Workspace

Branch: feature/project-purpose
Revision: 5c28c4a43872fb0bf24df43b5838889445e2d46a

## Progress

The user agreed the Purpose text shown in chat. Other sections are condensed by the lines the section adds.

## Verification

`just check --only lint` and `just check --only memory` pass on the working tree (AGENTS.md at 70 nonblank lines,
CLAUDE.md the same text); the commit gate follows.

## Blockers

None.

## Next action

Commit VAC 1, show the user the diff and the draft card on removing the lint ratchet; merge after the user's answer.
