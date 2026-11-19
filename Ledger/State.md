# State

## Focus

P146 (B147), VAC 1: `feature-start` on the base carries uncommitted changes to the new branch (D194).

## Workspace

Branch: feature/start-after-init
Revision: f7d197153f50876bc4afcbb0bae1f9a0bfaa965b

## Progress

- `start` no longer refuses a dirty tree; base, pending operation, name and existing-branch refusals remain.
- Tests: carrying changes, a pending merge and an invalid name refused; the delivery test runs `init` on the base and
  then `just feature-start bootstrap`.
- D194 supersedes D190's `git switch -c` consequence; setup README and example walkthrough updated.

## Verification

- `just run test -- change_delivery::git::tests::start`: 3 passed.
- `just run test -- project_setup::tests::delivery`: 2 passed.

## Blockers

None.

## Next action

Commit VAC 1 through the gate, then pin the head, archive P146 and B147 and run `just feature-merge`.
