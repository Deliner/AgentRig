# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: verify the full gate inside Mercurial commit transactions and reject mutated revision exports.

## Workspace

Branch: feature/vcs-backends

Revision: 3f089bb

P006 is integrated at 0c9cc39. Commit 3f089bb adds exact-revision gates, evidence and native parent memory history; its full gate passed with 459 Python and 113 Rust tests. Resume confirmed a clean branch without merge/rebase operations and a completed successful staged gate for the preceding HEAD. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. A real Mercurial pretxncommit hook can run the full check --revision command on HG_NODE, including its pending changeset. Failed checks roll back the transaction and partial commits preserve unselected working changes. The hook test exposed an error: mutated exports were marked inputs-changed but returned zero, allowing commit. Exact-revision gates now return an actionable error for this case using the existing fingerprint result. The manual registration is documented; automatic setup and branch guards remain required alongside shared delivery configuration, native branch/recovery facts, private adapters and integration/recovery. The isolated patch builder's internal Git dependency remains explicit. P008 is pending.

## Verification

The initial real Mercurial hook test proved failed-lint rollback and exposed acceptance of a mutated export. After the gate correction, all 33 Python Git/Mercurial delivery and evidence cases pass, including the reproducer, partial-commit preservation and existing Git/evidence behavior. Ruff, structural lint and diff whitespace checks pass. This VAC's full staged gate is pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the transaction-gate fix through the staged gate. Implement shared VCS delivery configuration and automatic native setup/registration, preserving existing hooks and using the verified pending-revision command for Mercurial. Continue native branch guards/recovery facts, private adapters and integration/recovery operations. Verify full independent Git/Mercurial consumer acceptance and private extension behavior before P007 integration, then deliver P008. Preserve the installed development pin while testing candidates.
