# State

## Focus

P196 (B203) VAC2: a merge requires compaction and the agent hook enforces it (AC1–AC6 in the runtime, D230).

## Workspace

Branch: feature/compact-after-merge
Revision: 7352c620e07666c82537682937a17016e7c23361

## Progress

- `feature-merge` records `compaction` (JSON `true`) in the runtime directory after a merge and ends its output with
  the instruction in `change_delivery/git/compaction.txt`.
- The hook refuses every tool with that instruction while it is due, and with the cause when unreadable; the tool
  `mcp__self-compact__compact` always passes. SessionStart `compact`, `clear` or `startup` lifts it.
- The record is protected; D230 narrows D200.

## Verification

`just run test -- change_delivery::git::tests::compaction` passed 3 tests on the first version; `just run lint` passed
after the record moved to the JSON helpers. The commit gate runs the affected groups.

## Blockers

None.

## Next action

Commit VAC2; then VAC3: ship the mod `self-compact` through `init` and into the rig with the user's approval.
