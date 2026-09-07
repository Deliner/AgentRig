# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: register VCS hooks through Source and the private protocol.

## Workspace

Branch: feature/vcs-backends

Revision: 8299bf2

P006 is integrated at 0c9cc39. Commit 8299bf2 adds shared initialization; its complete staged gate passed with 545 Python and 129 Rust tests. This VAC started from a clean feature/vcs-backends branch. No merge or rebase is pending; resume reports the previous gate completed successfully. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Source owns shared registration conflict and installed-setting checks. Setup validation, installation and doctor's VCS registration check dispatch through it. Private registration describes managed key/current/desired values, and register-hooks writes only within the repository before AgentRig checks the observed result. The independent Mercurial example reads native configuration, preserves user settings and appends only missing managed entries. Full private setup remains unavailable until file generation, repository discovery and harness/MCP command generation are implemented. Private commit and integration also remain required. The isolated patch builder still uses Git internally. P008 is pending.

## Verification

All 12 external-protocol and 3 registration Rust tests pass. Registration tests verify read-only inspection before initialization, actual native hook execution through a Mercurial commit, unchanged custom configuration, repeat registration, both managed-setting conflicts, invalid descriptions and false write success. All 22 focused Python native setup/doctor/Mercurial regressions pass. Structural lint passes after naming missing/unchanged conditions; formatting and diff whitespace checks pass. This VAC's full commit gate remains pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the registration VAC through the staged gate, correcting failures. Continue private setup: package adapters and hook/MCP generation still require native Kind, and preview still discovers native repositories. Bundle/input generation must receive the consumer root for configured relative adapter commands. Then implement private commit and integration around existing mandatory gates; temporary unimplemented errors are not P007 completion. Audit remaining metadata-only/Git callers, including index and commit-guard preparation. Complete independent installed-consumer acceptance for Git, Mercurial and the private extension. Release-template export belongs to the Git-only 0.2.0 to 0.3.0 migration. Verify all P007 acceptance before integration, then deliver P008. Preserve the installed development pin while testing candidates.
