# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: preserve committed decision history in Git and Mercurial memory checks.

## Workspace

Branch: feature/vcs-backends

Revision: d807aa9

P006 is integrated at 0c9cc39. Native VCS reads, selectable review snapshots, delegation and lint inventories are committed on feature/vcs-backends. The latest commit d807aa9 passed the full gate with 437 Python and 108 Rust tests. User remarks are retained in Ledger/Requests.md. The current VAC changes shared HEAD discovery and historical memory validation, tests and documentation. The installed development pin is unchanged.

## Progress

P007 is active. Review/delegation source snapshots and lint inventories use the shared Git/Mercurial boundary. Historical memory checks now use the native checked-out revision and tree instead of treating Mercurial as absent Git history. Published decision identities/details remain protected, including after moving memory directories. Unborn repositories have no baseline; backend failures propagate. Shared delivery configuration, setup, exact revision gates, private adapters and delivery/recovery operations remain required; the isolated patch builder's internal Git dependency remains explicit. P008 is pending.

## Verification

Thirteen Python memory tests and seven native VCS tests pass. Both real VCS backends reject changed decision identities, details and removed decisions through memory-check and a memory gate; relocating memory retains the committed baseline. Native HEAD discovery distinguishes unborn repositories from committed revisions and propagates backend format errors. The committed-table parsing extraction was rechecked with all four native-history Python scenarios. Structural lint has no blocking findings. This VAC's full staged gate is pending; no full P007 acceptance is claimed.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit native memory history through the staged gate. Implement exact-revision gates/evidence and backend hook semantics before enabling Mercurial setup; current gates still expose Git --staged and evidence still reads Git HEAD/index. Continue shared delivery configuration, private adapters and delivery/recovery operations, consolidating remaining VCS-specific code at the shared boundary. Verify full independent Git/Mercurial consumer acceptance and private extension behavior before P007 integration, then deliver P008. Preserve the installed development pin while testing candidates.
