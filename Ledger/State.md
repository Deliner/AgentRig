# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: own full revision and index exports in the shared VCS boundary.

## Workspace

Branch: feature/vcs-backends

Revision: 85e921a

P006 is integrated at 0c9cc39. The latest commit 85e921a adds native check freshness after reads, review, delegation, lint and memory history. Resume reports a clean branch without merge/rebase operations and a completed successful staged gate for the preceding HEAD; its revision differs after commit, so it does not claim current verification. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Full revision exports now use native Git/Mercurial trees and bytes, retain project settings, skills, executable modes and symlinks, and reject VCS control paths and unsupported submodules. Links are created after files to prevent writes through exported links. Gate index export now belongs to the shared VCS boundary; Git checkout-index semantics remain, and Mercurial explicitly has no index. This primitive is not yet exposed as check --revision. Outer resume branch/merge facts, shared delivery configuration, setup, exact revision gates, private adapters and delivery/recovery operations remain required. The isolated patch builder's internal Git dependency remains explicit. P008 is pending.

## Verification

Ten real-repository Rust VCS tests and fifteen Python delivery/evidence tests pass. They cover exact full exports despite dirty files, changed/deleted paths, binary bytes, modes and links, retained settings/skills, explicit unsupported exports, unchanged staged content and existing failed-commit/recovery behavior. The reported long test was split by file-kind assertions; structural lint now reports no errors. This VAC's full staged gate is pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the shared exports through the staged gate. Wire exact-revision checking, its evidence and history baseline, then native transaction hooks before Mercurial setup. Continue shared delivery configuration, native branch/recovery facts, private adapters and delivery/recovery operations. Verify full independent Git/Mercurial consumer acceptance and private extension behavior before P007 integration, then deliver P008. Preserve the installed development pin while testing candidates.
