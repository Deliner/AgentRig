# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: create native Git/Mercurial feature branches through the shared VCS owner.

## Workspace

Branch: feature/vcs-backends

Revision: 83e1b7d

P006 is integrated at 0c9cc39. Commit 83e1b7d adds native recovery observations and read-only Mercurial queries; its complete staged gate passed with 473 Python and 113 Rust tests. Resume confirms no merge/rebase and a completed successful preceding gate. The current changes implement feature-start. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Feature-start now selects the native repository, requires the configured base, clean files and no pending merge/rebase, and creates a Git reference or Mercurial named branch. Mercurial writes honor consumer hooks and native branch names; the name becomes persistent with its next commit. Native feature-merge, backend-owned configuration naming validation, private adapters and complete independent consumer acceptance remain required. The isolated patch builder's internal Git dependency remains explicit. P008 is pending.

## Verification

All 21 test_git.py cases pass: existing Git integration/commit checks, Mercurial setup and transactions, native feature creation and parent preservation, dirty/untracked file rejection, invalid and existing names, wrong starting branch, consumer hook rejection and a pending Mercurial merge with clean files. Structural lint reported no errors; Ruff formatting passes. This VAC's full staged gate is pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the native feature-start VAC through the staged gate, correcting any failures. Continue native integration operations and backend-owned configuration naming validation, then private adapters. Review upgrade backend propagation as part of recovery acceptance. Verify full independent Git/Mercurial consumer acceptance and private extension behavior before P007 integration, then deliver P008. Preserve the installed development pin while testing candidates.
