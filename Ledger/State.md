# State

## Focus

P156 (B157): agents change enforcement files only through the user's approval (D213). VAC1 of three: the
protection set and the hook refusal of edits and `just write` operands that reach it (AC1).

## Workspace

Branch: feature/enforcement-files
Revision: 6078b08f4cc3f4d0f5216e977704e012be6b21a6

## Progress

- VAC1 adds `project_config/protection.rs`, the settings `hooks.protected` and `git.pin`, the shared refusal for
  edits and project-access catalog operands, Git operands of rm, mv, restore and checkout, a resolve error that
  refuses with its cause, the recovery split of readable protected modules, the propose-protected-change skill and D213.
- Next VACs: `just approve` with the approval record and init approvals, the gate check of approved content and the
  pin ancestry (AC2); blocking on unavailable route state (AC3).

## Verification

- `just run test -- agent_hooks`: 43 passed, including five protected tests for B1-B4, B6 and A8.
- `just run test -- project_`: 110 passed. `just run clippy`, `just run rustfmt`, `just lint` and
  `just check --only memory` pass.

## Blockers

None.

## Next action

Commit VAC1, then implement `agentrig approve`, its recipe and the hook refusal of agent approvals.
