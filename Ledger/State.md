# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: select Git/Mercurial for snapshots and isolated review, including repair reviews.

## Workspace

Branch: feature/vcs-backends

Revision: 134cf94

P006 is integrated at 0c9cc39. The first P007 component is committed at 134cf94 through the full gate (430 Python and 103 Rust tests). The current branch retains that component and the installed development pin is unchanged. Ledger/Requests.md separately preserves the user's requested reminder and original description; it is untracked and must be preserved.

## Progress

P007 is active. Mercurial 7.2.4 is supplied by the locked uv test environment. Native revision/file reads are committed. Review project YAML now selects repository.vcs (git by default, or mercurial). Snapshot export, visibility checks and repair-review diffs use that backend; Mercurial control metadata is excluded. The current delegate scope still defaults to Git. Shared delivery configuration, setup, private adapters, delegation and delivery/recovery operations remain required. P008 is pending.

## Verification

Nineteen focused Rust tests pass: five native VCS/config/snapshot tests, three existing snapshot tests and eleven execution tests. A real Mercurial repository completes isolated review and repair review through bubblewrap with a test critic; original base, changed candidate, repair diff and dirty worktree preservation are verified. No real model acceptance is claimed. The fixture extraction was rechecked with the Mercurial execution test, and structural lint has no blocking findings. This VAC's full staged gate is pending; the prior component's passing gate does not prove this content.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit snapshot/review backend selection through the staged gate. Continue shared project configuration, setup, delegation, private adapters and all configured delivery/recovery operations with actual backend capabilities. Commit the separately requested reminder without losing it. Verify independent Git/Mercurial consumers and private extension behavior before P007 acceptance/integration, then deliver P008. Preserve the installed development pin while testing candidates.
