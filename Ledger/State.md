# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: generate private VCS files and complete setup/preview/doctor routing.

## Workspace

Branch: feature/vcs-backends

Revision: 5b36ee6

P006 is integrated at 0c9cc39. Commit 5b36ee6 adds shared registration; its complete staged gate passed with 545 Python and 132 Rust tests. This VAC started from a clean feature/vcs-backends branch. No merge or rebase is pending; resume reports the previous gate completed successfully. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Backend generation owns VCS hook/runtime-ignore files and the root command embedded in harness/MCP launchers. Private generate runs read-only from the consumer root and validates returned file paths within the hook area. Bundle/input preparation carries that root through external configuration loading. Setup, preview and doctor now route private generation, repository presence, initialization and registration; init CLI choices remain native. The independent Mercurial example implements these operations. Private commit guards and integration remain unsupported and required, so successful installation does not establish full delivery acceptance. The isolated patch builder still uses Git internally. P008 is pending.

## Verification

All 5 generation/registration Rust tests pass, including native/external Mercurial byte equality, no writes during generation, escaped-file rejection and invalid root commands. Fourteen focused Python setup scenarios pass: native/private install and repeat, preserved settings, real generated review MCP handshake, preview and asset/harness conflicts. A relative adapter resolves from the consumer root when the CLI runs elsewhere; the installed binary passes doctor and repeat preview. Candidate build, structural lint, formatting and diff whitespace checks pass. This VAC's full commit gate remains pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the generation/setup VAC through the staged gate, correcting failures. Implement private commit guards and integration around existing mandatory gates; temporary unimplemented errors are not P007 completion. Audit remaining metadata-only/Git callers, including index and commit-guard preparation, and test configuration update/rollback through private setup. Complete independent installed-consumer delivery acceptance for Git, Mercurial and the private extension. Release-template export belongs to the Git-only 0.2.0 to 0.3.0 migration. Verify all P007 acceptance before integration, then deliver P008. Preserve the installed development pin while testing candidates.
