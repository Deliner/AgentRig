# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: verify configuration update and rollback in installed Git and Mercurial consumers.

## Workspace

Branch: feature/vcs-backends

Revision: 89f6c73

P006 is integrated at 0c9cc39. Commit 89f6c73 selects private sources for review and delegation; its complete staged gate passed with 526 Python and 123 Rust tests. Resume confirms no merge/rebase, a clean starting tree and a completed successful preceding gate. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Backend selects native Kind or an external Adapter declaration; Source dispatches reads into existing snapshot checks. Review repository.vcs and delegation vcs accept command objects while retaining native string serialization and diagnostics. Existing report configuration and task inputs retain source identity. Re-review rejects changed selection and, for opaque external IDs, a different canonical root; no new identity registry was added. Root normalization stays within persisted technical-error handling. Private project setup/delivery selection, writes, configured inventory/history/evidence and full consumer acceptance remain required. The isolated patch builder still uses Git internally. P008 is pending.

## Verification

All six configuration-upgrade tests pass. The existing update/rollback acceptance now runs against installed Git and Mercurial consumers with custom service paths. It verifies changed command execution, updated skills, preserved memory, selected backend and byte-identical native hook registration, then restored installation bytes and command behavior after rollback. No production change was needed for this path. This VAC's full staged gate is pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the upgrade-acceptance VAC through the staged gate, correcting any failures. Extend project VCS selection and operations to private adapters around existing setup, commit, integration and recovery gates. Replace metadata-only discovery where it ignores configured sources, including inventory/history/evidence. Review remaining Git-specific callers and independent installed-consumer acceptance. Release-template export currently belongs to the Git-only 0.2.0 to 0.3.0 migration; do not infer that its old CLI accepts --vcs. Verify all P007 acceptance before integration, then deliver P008. Preserve the installed development pin while testing candidates.
