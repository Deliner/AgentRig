# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: verify private configuration upgrades, local conflicts and exact rollback through consuming projects.

## Workspace

Branch: feature/vcs-backends

Revision: f27a241

P006 is integrated at 0c9cc39. Commit f27a241 adds private integration; its full staged gate passed, including 560 Python tests. This VAC started from a clean feature/vcs-backends branch. No merge or rebase is pending; resume reports the previous gate completed successfully. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Private integration is committed. The existing configuration upgrade consumer now selects the private adapter before setup and verifies application, local conflict preservation and rollback alongside native backends. No production upgrade changes were needed. The isolated patch builder still uses Git internally. Full installed delivery acceptance and staging-index selection audit remain required; P008 is pending.

## Verification

All eight configuration upgrade tests pass, including private command/skill updates, retained consumer State and hook registration, exact snapshot rollback, and blocked local conflicts followed by explicit keep resolution. An initial test import error was corrected. Structural lint passes. This VAC's full commit gate remains pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the private upgrade acceptance VAC through the staged gate, correcting failures. Audit remaining metadata-only/Git callers: gate index export and evidence index fingerprint still discover metadata; preserve the existing staged-configuration contract when making capability selection explicit. Complete independent installed-consumer delivery acceptance for Git, Mercurial and the private extension. Release-template export belongs to the Git-only 0.2.0 to 0.3.0 migration. Verify all P007 acceptance before integration, then deliver P008. Preserve the installed development pin while testing candidates.
