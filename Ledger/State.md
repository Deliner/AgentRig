# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: bind check freshness to native Git/Mercurial revisions and inventories.

## Workspace

Branch: feature/vcs-backends

Revision: 361873c

P006 is integrated at 0c9cc39. Native VCS reads, selectable review snapshots, delegation, lint inventories and historical memory validation are committed on feature/vcs-backends. The latest commit 361873c passed the full gate with 441 Python and 109 Rust tests. User remarks are retained in Ledger/Requests.md. The current VAC changes evidence revision/inventory reads and stages Git index access at the shared VCS boundary. The installed development pin is unchanged.

## Progress

P007 is active. Review/delegation snapshots, lint inventories and historical memory checks use native Git/Mercurial reads. Check evidence now obtains revisions and working files through that same boundary, so Mercurial can prove or invalidate freshness without including its control metadata. Git index files/entries are owned by the VCS module; Mercurial rejects --staged explicitly. Outer resume branch/merge facts, shared delivery configuration, setup, exact revision gates, private adapters and delivery/recovery operations remain required. The isolated patch builder's internal Git dependency remains explicit. P008 is pending.

## Verification

Ten Python evidence tests pass, covering both Git and Mercurial under I018 plus existing staged-index checks, repeats and recovery. Mercurial evidence becomes stale on content, executable-mode or revision changes; metadata/ignored-file changes do not alter source fingerprints. An advanced revision with restored original bytes matches content but not revision, and --staged fails explicitly. The only structural finding was a negated test condition, now named without changing behavior. This VAC's full staged gate is pending; no full P007 acceptance is claimed.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit native check evidence through the staged gate. Implement exact-revision gate exports and backend hook semantics before enabling Mercurial setup; current gate export remains Git checkout-index and no --revision mode exists. Continue shared delivery configuration, native branch/recovery facts, private adapters and delivery/recovery operations. Verify full independent Git/Mercurial consumer acceptance and private extension behavior before P007 integration, then deliver P008. Preserve the installed development pin while testing candidates.
