# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: initialize repositories through the shared Backend and private protocol.

## Workspace

Branch: feature/vcs-backends

Revision: ad4d0fa

P006 is integrated at 0c9cc39. Commit ad4d0fa adds private feature-start; its complete staged gate passed with 545 Python and 127 Rust tests. This VAC started from a clean feature/vcs-backends branch. No merge or rebase is pending; resume reports the previous gate completed successfully. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Backend now dispatches initialization to native or external implementations, and setup installation calls that owner. The independent Mercurial adapter initializes absent repositories, preserves existing ones and refuses Git metadata. Initialization and feature-start share native writes honoring consumer configuration and hooks; reads retain their read-only sandbox. Initialization permits repository writes and requires readable observation afterward. Full private setup remains unavailable until file generation, repository discovery and hook/MCP registration are implemented. Private commit and integration also remain required. The isolated patch builder still uses Git internally. P008 is pending.

## Verification

All 12 external-protocol Rust tests pass. The new cases initialize real Git, Mercurial and external Mercurial repositories with existing user files, repeat initialization without changing branch or any file bytes, preserve a dirty committed external repository and reject a Git repository without mutation. All 7 focused Python setup/feature-start regression cases pass. Structural lint passes. This VAC's full commit gate remains pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the initialization VAC through the staged gate, correcting failures. Continue private setup: package adapters and hook/MCP generation still require native Kind, while registration, preview and doctor still use native repositories. Then implement private commit and integration around existing mandatory gates; temporary unimplemented errors are not P007 completion. Audit remaining metadata-only/Git callers, including index and commit-guard preparation. Complete independent installed-consumer acceptance for Git, Mercurial and the private extension. Release-template export belongs to the Git-only 0.2.0 to 0.3.0 migration. Verify all P007 acceptance before integration, then deliver P008. Preserve the installed development pin while testing candidates.
