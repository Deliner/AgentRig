# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: select private sources for direct lint commands in both binaries.

## Workspace

Branch: feature/vcs-backends

Revision: e8f66bd

P006 is integrated at 0c9cc39. Commit e8f66bd selects private project reads and recovery; its complete staged gate passed with 535 Python and 126 Rust tests after fixture repairs. This VAC started from a clean feature/vcs-backends tree. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Project reads and recovery use the selected Backend/Source. Direct lint, lint-config-check and lint-explain in both binaries now accept --vcs-config FILE with the existing Backend YAML declaration. Reruns retain the absolute selection-file path. No option preserves native discovery/physical inspection; direct lint does not implicitly load project VCS settings. The project gate supplies its own source and handles exports separately. Private setup, hook/MCP generation and delivery still return explicit unimplemented-operation errors. The isolated patch builder still uses Git internally. P008 is pending.

## Verification

All 19 focused inventory, explain and portable-linter tests pass. The private adapter is exercised against real Mercurial through both binaries for lint, configuration validation and explain. Results and executable reruns agree; ignored/deleted/link/control files stay excluded, and all consumer file bytes including metadata stay unchanged. Invalid declarations and unsupported adapters fail without native fallback. Structural lint passes after naming the inspection condition and separating finding/rerun assertions and path normalization. This VAC's full gate remains pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the direct-lint-selection VAC through the staged gate, correcting failures. Implement private setup, registration, commit and integration operations around existing mandatory gates; explicit temporary unimplemented errors are not P007 completion. Audit remaining metadata-only/Git callers, including index and commit-guard preparation. Complete independent installed-consumer acceptance for Git, Mercurial and the private extension. Release-template export belongs to the Git-only 0.2.0 to 0.3.0 migration. Verify all P007 acceptance before integration, then deliver P008. Preserve the installed development pin while testing candidates.
