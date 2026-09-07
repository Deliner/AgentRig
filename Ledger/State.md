# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: export exact private revisions through the shared delivery-input exporter.

## Workspace

Branch: feature/vcs-backends

Revision: 6135a5b

P006 is integrated at 0c9cc39. Commit 6135a5b verifies Git/Mercurial configuration updates and rollback; its complete staged gate passed with 527 Python and 123 Rust tests. This VAC started from a clean feature/vcs-backends tree. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Backend selects native Kind or an external Adapter declaration; Source dispatches reads into existing snapshot checks. Review repository.vcs and delegation vcs accept command objects while retaining native string serialization and diagnostics. Existing report configuration and task inputs retain source identity. Re-review rejects changed selection and, for opaque external IDs, a different canonical root; no new identity registry was added. Root normalization stays within persisted technical-error handling. Private project setup/delivery selection, writes, configured inventory/history/evidence and full consumer acceptance remain required. The isolated patch builder still uses Git internally. P008 is pending.

## Verification

All 19 focused native/external VCS tests pass. Source::export_revision now resolves once and uses the existing full-input exporter for native and external reads. Real Mercurial through the separate adapter preserves committed binary bytes, symlinks, executable mode and dirty source contents; synthetic protocol replies prove opaque IDs remain fixed and unsafe paths, submodules and conflicting link trees are rejected. Existing native exact/index export tests pass. This VAC's full staged gate is pending; the project CLI still does not select private backends, so full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the exact-export VAC through the staged gate, correcting any failures. Extend project VCS selection and operations to private adapters around existing setup, commit, integration and recovery gates; Source now has the full revision export required by check --revision. Replace metadata-only discovery where it ignores configured sources, including inventory/history/evidence. Review remaining Git-specific callers and independent installed-consumer acceptance. Release-template export currently belongs to the Git-only 0.2.0 to 0.3.0 migration; do not infer that its old CLI accepts --vcs. Verify all P007 acceptance before integration, then deliver P008. Preserve the installed development pin while testing candidates.
