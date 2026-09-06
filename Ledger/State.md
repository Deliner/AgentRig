# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: obtain lint inventories through the shared Git/Mercurial owner.

## Workspace

Branch: feature/vcs-backends

Revision: 94490c2

P006 is integrated at 0c9cc39. Native VCS reads, selectable review snapshots and delegation are committed on feature/vcs-backends. User remarks and recovery documentation are committed at 94490c2 through the full gate with 435 Python and 107 Rust tests. The current VAC changes lint inventory only, its shared VCS operations, tests and documentation. The installed development pin is unchanged.

## Progress

P007 is active. Review and delegation select Git/Mercurial for exact source revisions; retained inputs/code reports identify that VCS. Lint now discovers Git/Mercurial at its selected root and obtains tracked plus non-ignored untracked paths from the shared owner. Mercurial uses .hgignore; hgrc-configured extra ignores remain disabled with user/repository configuration. Ambiguous roots and native inventory errors do not fall back to filesystem traversal. Plain directories still work. Shared delivery configuration, setup, private adapters and delivery/recovery operations remain required; the isolated patch builder's internal Git dependency remains explicit. P008 is pending.

## Verification

Five focused Python tests pass for both CLIs on Git, Mercurial and plain directories. Native VCS tests passed except an invalid fixture assumption: Mercurial accepts an empty .hg as an old format. The corrected fixture declares an unsupported format requirement and its focused retry passes, proving inventory errors propagate. The five other native VCS tests passed unchanged. Structural lint has no blocking findings. This VAC's full staged gate is pending; no full P007 acceptance is claimed.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit native lint inventory through the staged gate. Continue shared project configuration, setup, private adapters and all configured delivery/recovery operations with actual backend capabilities, consolidating remaining VCS-specific code at the shared boundary. Setup still initializes Git and configures Git hooks directly; route that behavior with actual backend hook/gate semantics. Verify full independent Git/Mercurial consumer acceptance and private extension behavior before P007 integration, then deliver P008. Preserve the installed development pin while testing candidates.
