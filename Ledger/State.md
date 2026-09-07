# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: validate configured branch names through the selected VCS owner.

## Workspace

Branch: feature/vcs-backends

Revision: 50853e0

P006 is integrated at 0c9cc39. Commit 50853e0 consolidates native integration; its complete staged gate passed with 491 Python and 113 Rust tests. Resume confirms no merge/rebase and a completed successful preceding gate. Current changes move configuration naming checks into the VCS owner. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Common configuration now delegates base/prefix validation to VCS settings. Git retains its existing reference restrictions; Mercurial follows native label rules and rejects names its CLI would normalize. Its configured base and prefix can contain internal spaces. Naming validation creates no repository and runs no native commands. Private adapters and full independent consumer acceptance remain required. The isolated patch builder's internal Git dependency remains explicit. P008 is pending.

## Verification

All 58 setup/configuration tests pass, including 28 backend/name combinations and a real Mercurial setup, repeat setup and hook-checked commit with spaces in configured names. Corrected the new test's bootstrap order; product clean-tree rules remain unchanged. Structural lint and whitespace checks pass. This VAC's full staged gate is pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the backend naming VAC through the staged gate, correcting any failures. Implement the private adapter contract and separate consumer example. Review remaining Git-specific callers, upgrade backend propagation and independent installed-consumer integration as part of full recovery acceptance. Verify all Git/Mercurial acceptance and private extension behavior before P007 integration, then deliver P008. Preserve the installed development pin while testing candidates.
