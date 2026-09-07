# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: observe native Git/Mercurial recovery facts through resume without changing repository files.

## Workspace

Branch: feature/vcs-backends

Revision: 363c2c7

P006 is integrated at 0c9cc39. Commit 363c2c7 adds native setup and registration; its complete staged gate passed with 467 Python and 113 Rust tests. Resume confirmed clean Git state, no merge/rebase and matching content from that gate, with the expected changed HEAD after commit. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Resume now reports a native vcs observation, preserving the git alias for Git consumers, and resolves recorded hashes through the same repository. Actual merge/rebase state is observed for both backends, including linked Git worktrees. Mercurial reads use bubblewrap read-only mounts after tests exposed cache and dirstate writes during native inspection; initialization remains a separate native write. Setup preview and doctor describe this dependency. Native feature-start/feature-merge, backend naming validation, private adapters and complete independent consumer acceptance remain required. The isolated patch builder's internal Git dependency remains explicit. P008 is pending.

## Verification

All 10 focused resume cases pass, including real Git/Mercurial merge/rebase conflicts, abbreviated State revisions, plain and unborn repositories, linked worktrees and exact file nonmutation. All 47 Python feedback, Git/Mercurial transaction and VCS delegation cases pass. All 22 Rust VCS/review execution cases pass. The full gate passed 472 Python cases but exposed an inherited GIT_INDEX_FILE in the new linked-worktree fixture before resume ran. The fixture now clears that variable; its exact environment reproducer passes. Structural lint and whitespace checks pass. The corrected full staged gate is pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the native recovery observation VAC through the staged gate, correcting any failures. Continue native branch/integration operations and backend-owned naming validation, then private adapters. Review upgrade backend propagation as part of recovery acceptance. Verify full independent Git/Mercurial consumer acceptance and private extension behavior before P007 integration, then deliver P008. Preserve the installed development pin while testing candidates.
