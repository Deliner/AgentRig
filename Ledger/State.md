# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: create feature branches through the private VCS protocol.

## Workspace

Branch: feature/vcs-backends

Revision: 6f5e0f8

P006 is integrated at 0c9cc39. Commit 6f5e0f8 selects private sources in both direct linter interfaces; its complete staged gate passed with 541 Python and 126 Rust tests. The interrupted feature-start changes were inspected and completed in this VAC. No merge or rebase is pending. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Source owns the common feature-start preconditions and dispatches native or external branch creation. The private start-feature operation receives the branch and expected state, runs with only the repository writable, and is followed by observation to reject false success. The independent Mercurial example rechecks state and honors native hooks. Reads retain their read-only sandbox. Private setup, hook/MCP generation, commit and integration remain required. The isolated patch builder still uses Git internally. P008 is pending.

## Verification

All 15 focused feature-start/private-selection Python cases pass, including real Git/Mercurial and external Mercurial success, untracked-file preservation, invalid names and native hook refusal. The first run exposed a fixture reading through its own rejecting hook; observation through resume repaired that fixture. All 10 external-protocol Rust tests pass, including false write success rejection and unchanged read isolation/export restrictions. Structural lint and diff whitespace checks pass. This VAC's full commit gate remains pending; full P007 acceptance remains unproven.

The first commit gate stopped at mypy because the fixture imported resumed indirectly. Importing it from its defining module repairs the issue; the staged mypy rerun passes. The complete commit gate must still pass.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the feature-start VAC through the staged gate, correcting failures. Implement private setup, registration, commit and integration operations around existing mandatory gates; explicit temporary unimplemented errors are not P007 completion. Audit remaining metadata-only/Git callers, including index and commit-guard preparation. Complete independent installed-consumer acceptance for Git, Mercurial and the private extension. Release-template export belongs to the Git-only 0.2.0 to 0.3.0 migration. Verify all P007 acceptance before integration, then deliver P008. Preserve the installed development pin while testing candidates.
