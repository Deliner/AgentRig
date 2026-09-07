# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: implement and verify the private VCS read process boundary and independent example.

## Workspace

Branch: feature/vcs-backends

Revision: 07a270f

P006 is integrated at 0c9cc39. Commit 07a270f delivers backend-owned naming; its complete staged gate passed with 520 Python and 113 Rust tests. Resume confirms no merge/rebase and a completed successful preceding gate. Current changes add the private read protocol, example and tests. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. The external Adapter Rust API invokes a separately supplied command with a versioned JSON read request under read-only bubblewrap. Typed replies cover exact revisions, parents, tree entries, bytes, changed/working paths and diff; unknown/invalid data and unsupported operations are explicit errors. Opaque IDs do not require Git hash syntax. An independent Python example reads real Mercurial without AgentRig imports. Shared export-path validation rejects noncanonical aliases and control paths. Project/review/delegate selection, private delivery writes and full consumer acceptance remain required. The isolated patch builder's internal Git dependency remains explicit. P008 is pending.

## Verification

All 18 focused external/native VCS and snapshot tests pass. Four new cases exercise the real example with binary files, symlinks, renamed paths and dirty work; compare repository files including metadata before/after; reject malformed replies, invalid versions/types/IDs/paths and duplicate entries; and prove an attempted working-file overwrite fails. The full staged attempt passed 520 Python and 117 Rust tests, then review Clippy rejected a redundant test clone. Replaced it with a borrowed slice; the focused staged Clippy check passes. A complete commit retry remains pending. Full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the private read-protocol VAC through the staged gate, correcting any failures. Wire a configured external backend through the common VCS owner and project/review/delegate selection; add private setup, integration and recovery operations around existing gates. Replace metadata-only discovery where it ignores configured private backends. Review remaining Git-specific callers, upgrade backend propagation and independent installed-consumer acceptance. Verify all P007 acceptance before integration, then deliver P008. Preserve the installed development pin while testing candidates.
