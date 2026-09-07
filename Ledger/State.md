# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: consolidate Git and Mercurial integration through the VCS owner and common checked-content gate.

## Workspace

Branch: feature/vcs-backends

Revision: 87a7e4e

P006 is integrated at 0c9cc39. Commit 87a7e4e delivers Mercurial integration; its complete staged gate passed with 490 Python and 113 Rust tests. Current resume confirms no merge/rebase and identifies the previous State as stale. Current changes move Git integration into the VCS module. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Both native backends now enter integration through one VCS interface. Git retains merge-preserving rebase and the existing runtime command executor; the project workflow owns the full gate and process cleanup. Both backends reject changed gate inputs before completing integration. Mercurial retains native pending-parent and transaction recovery. Backend-owned configuration naming, private adapters and full independent consumer acceptance remain required. The isolated patch builder's internal Git dependency remains explicit. P008 is pending.

## Verification

All 60 existing Git/Mercurial delivery and feedback tests pass. The new Git regression also passes: a successful check that changes a file stops integration, preserving both branches and the changed file. Existing coverage includes retained merge history, conflict recovery, gate failures, Mercurial transaction refusal and owned-process cleanup. Structural lint and whitespace checks pass. This VAC's full staged gate is pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the common integration VAC through the staged gate, correcting any failures. Implement backend-owned configuration naming, then private adapters. Review remaining Git-specific callers, upgrade backend propagation and independent installed-consumer integration as part of full recovery acceptance. Verify all Git/Mercurial acceptance and private extension behavior before P007 integration, then deliver P008. Preserve the installed development pin while testing candidates.
