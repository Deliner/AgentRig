# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: share native Git/Mercurial revision and file reads.

## Workspace

Branch: feature/vcs-backends

Revision: 0c9cc39

Git confirms P006 merged into master at 0c9cc39 after completion and integration gates passed with 430 Python and 101 Rust tests. The old feature branch is retained. P007 starts from that clean base on feature/vcs-backends. No unrelated work is present; the installed development pin remains unchanged.

## Progress

P007 is active. Mercurial 7.2.4 is selected and installed as a locked uv test dependency. A shared vcs owner in the existing native library implements Git/Mercurial revision resolution, tree entries, raw file reads, changed paths and diffs. Existing Git snapshot helpers now use that owner. Mercurial reads ignore user/repository configuration and require a single resolved revision. YAML backend selection, private adapters, workflow routing and delivery/recovery operations remain required. P008 is pending.

## Verification

Five focused Rust tests pass: two real-backend tests and three existing snapshot tests. They verify exact committed bytes despite dirty worktrees, binary files, executable/symlink kinds, changed/deleted paths and diffs, ambiguous Mercurial revsets and disabled repository aliases/hooks. Structural lint has no blocking findings. The first P007 VAC's full staged gate remains pending. No complete P007 acceptance is claimed.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit shared native VCS reads through the staged gate. Add explicit backend selection and route snapshots/review/delegation through it; implement the private adapter contract and all configured delivery/recovery operations with actual backend capabilities. Verify independent Git/Mercurial consumers and private extension behavior before P007 acceptance/integration, then deliver P008. Preserve the installed development pin while testing candidates.
