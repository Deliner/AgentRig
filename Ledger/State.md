# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: merge Mercurial features with checked content and native conflict/transaction recovery.

## Workspace

Branch: feature/vcs-backends

Revision: d0b0a64

P006 is integrated at 0c9cc39. Commit d0b0a64 adds native feature-start; its complete staged gate passed with 483 Python and 113 Rust tests. Resume confirms no merge/rebase and a completed successful preceding gate. Current changes implement Mercurial integration and tests. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Mercurial feature-merge selects a single base head, merges the committed feature, runs the full working-tree gate, verifies unchanged evidence and parents, and commits with native hooks. Native pending parents retain recovery after conflicts, failed checks and rejected transactions; no additional journal was introduced. Empty feature names, dirty worktrees and multiple base heads are rejected before switching. Remaining Git integration orchestration still needs the common VCS boundary, alongside backend-owned configuration naming, private adapters and full independent consumer acceptance. The isolated patch builder's internal Git dependency remains explicit. P008 is pending.

## Verification

All 59 Git/Mercurial delivery and feedback tests pass, including successful integration after a failed gate or real conflict, exact parent retention, preserved feature history, rejected mutated check inputs and pre-switch failures. The added commit-hook refusal case and gate-refusal case also pass independently, proving retry after transaction rollback. Structural lint and whitespace checks pass. This VAC's full staged gate is pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the Mercurial integration VAC through the staged gate, correcting any failures. Consolidate remaining Git integration orchestration at the VCS boundary and implement backend-owned configuration naming, then private adapters. Review upgrade backend propagation and independent installed-consumer integration as part of full recovery acceptance. Verify all Git/Mercurial acceptance and private extension behavior before P007 integration, then deliver P008. Preserve the installed development pin while testing candidates.
