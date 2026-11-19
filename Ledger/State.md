# State

## Focus

P156 (B157, D213) VAC2: the user's `just approve` applies a proposal of enforcement files and records the approved
content; the commit gate refuses unapproved protected content and a pin that does not move forward (AC1, AC2).

## Workspace

Branch: feature/enforcement-files
Revision: 26b6b72081a9be1562964e1529abed2b136cf73f

## Progress

- `approve` command, recipe and agent-hook refusal; `change_delivery/git/approval.rs` applies the proposal through
  `shared/atomic_files/overlay.rs`, restores it when the configuration fails to load and records the content
  `git add` would stage, computed in a temporary index (`Repository::stageable`), so the user's index stays as it was.
- The feature-branch commit guard checks approvals before State; a committed pin moves only to a descendant.
- `init` records its protected files; `bootstrap.sh` refuses a revision that does not descend from the pin.
- Delivery tests that change the catalog on a branch approve it first, as the user would.

## Verification

- `just run test -- change_delivery agent_hooks shared`: 117 passed; project_setup, cli_dispatch and atomic tests pass.
- clippy, rustfmt and lint pass.

## Blockers

None.

## Next action

Commit VAC2, then VAC3: unavailable or corrupt route state blocks a call like an unapplied required skill (AC3).
