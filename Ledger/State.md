# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: guard private VCS commits using the selected source and exact pending revision.

## Workspace

Branch: feature/vcs-backends

Revision: 2e894d3

P006 is integrated at 0c9cc39. Commit 2e894d3 adds private setup generation; its complete staged gate passed with 551 Python and 134 Rust tests. This VAC started from a clean feature/vcs-backends branch. No merge or rebase is pending; resume reports the previous gate completed successfully. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Source dispatches commit context to native or private backends. Guard preparation selects the configured source, resolves its reference and uses the same exact revision for exported configuration and commit context. Shared branch/merge policy is unchanged. The Mercurial example inspects the pending changeset and requires a revision; installed hooks run the separate mandatory revision check. Private integration remains unsupported and required. The isolated patch builder still uses Git internally. P008 is pending.

## Verification

All 13 external-protocol Rust tests pass, including typed commit-context rejection. Six focused Python cases pass. Real native/private installed hooks reject direct base commits and permit feature commits. A failing mandatory check rolls back the private Mercurial commit while retaining source edits; a subsequent selected commit succeeds without committing unrelated working changes. Three synthetic cases without Git/Mercurial metadata prove exact opaque-ID export/context routing and feature/base/merge policy. Structural lint, formatting and diff whitespace checks pass. This VAC's full commit gate remains pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the private commit-guard VAC through the staged gate, correcting failures. Implement private feature integration around existing mandatory gates and recoverable VCS state; temporary unimplemented errors are not P007 completion. Audit remaining metadata-only/Git callers, including staging-index preparation, and test configuration update/rollback through private setup. Complete independent installed-consumer delivery acceptance for Git, Mercurial and the private extension. Release-template export belongs to the Git-only 0.2.0 to 0.3.0 migration. Verify all P007 acceptance before integration, then deliver P008. Preserve the installed development pin while testing candidates.
