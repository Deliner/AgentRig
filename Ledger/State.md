# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: select private sources for project checks, historical memory, evidence and recovery.

## Workspace

Branch: feature/vcs-backends

Revision: 820586c

P006 is integrated at 0c9cc39. Commit 820586c exports exact private revisions; its complete staged gate passed with 527 Python and 125 Rust tests. This VAC started from a clean feature/vcs-backends tree. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Project Settings.backend now uses the shared Backend declaration. Exact checks, historical memory, content evidence, gate inventory and embedded lint use its Source; exported checks inspect physical exported inputs. Resume uses the new read-only observe operation and resolves opaque recorded IDs. Native selections require matching metadata. Native setup behavior is preserved; private setup, hook/MCP generation and delivery return explicit unimplemented-operation errors rather than pretending to support writes. Standalone lint still needs configured private selection. The isolated patch builder still uses Git internally. P008 is pending.

## Verification

The revision/history/evidence selection run passed 30 cases; repaired memory-location cases and three new private CLI tests pass. All nine external protocol tests pass, including nonmutating observation and invalid replies. Structural lint passes. The first full gate passed 530 Python cases and found five fixture failures: four empty Git commits after redundant backend configuration, and a quoted YAML replacement in the legacy-memory migration test. Git fixtures now retain default selection; relocation uses structured configuration editing. All nine focused repair/native-variant cases pass. The full staged gate must be retried; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the configured project-read VAC through the staged gate, correcting failures. Implement private setup, registration, commit and integration operations around existing mandatory gates; explicit temporary unimplemented errors are not P007 completion. Finish standalone lint selection and audit remaining metadata-only/Git callers, including index and commit-guard preparation. Complete independent installed-consumer acceptance for Git, Mercurial and the private extension. Release-template export belongs to the Git-only 0.2.0 to 0.3.0 migration. Verify all P007 acceptance before integration, then deliver P008. Preserve the installed development pin while testing candidates.
