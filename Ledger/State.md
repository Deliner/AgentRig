# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: select Git or Mercurial in project configuration and install native hooks through init/setup.

## Workspace

Branch: feature/vcs-backends

Revision: 3667e96

P006 is integrated at 0c9cc39. Commit 3667e96 rejects mutated revision exports during Mercurial commit transactions. The current working changes add native setup and registration, tests and documentation. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Canonical vcs settings retain the old git alias; CLI and wizard select the backend. Setup initializes the native repository, preserves custom hooks and ignore rules, registers a Mercurial pending-changeset guard/full gate and excludes runtime files. Generated MCP launchers and review presets select that backend. Mercurial feature-start/feature-merge currently return explicit unsupported errors; native delivery, recovery facts, private adapters and complete independent consumer acceptance remain required. The isolated patch builder's internal Git dependency remains explicit. P008 is pending.

## Verification

The previous focused-test handle was no longer available on resume. A fresh run of test_git.py, test_setup.py and test_package.py passed all 63 cases, including actual Mercurial commit rejection/success, repeat setup, custom registration preservation, review presets and both wizard backends. Structural lint reports no errors. This VAC's full staged gate is pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Finish formatting and commit the native setup VAC through the staged gate, correcting any failures. Continue native branch/integration/recovery operations and backend-owned naming validation, then private adapters. Review upgrade backend propagation as part of recovery acceptance. Verify full independent Git/Mercurial consumer acceptance and private extension behavior before P007 integration, then deliver P008. Preserve the installed development pin while testing candidates.
