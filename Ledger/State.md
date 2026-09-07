# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: private feature integration through shared mandatory checks and recoverable native state.

## Workspace

Branch: feature/vcs-backends

Revision: 316ad54

P006 is integrated at 0c9cc39. Commit 316ad54 adds private commit guards; its full staged gate passed, including 556 Python tests. This VAC started from a clean feature/vcs-backends branch. No merge or rebase is pending; resume reports the previous gate completed successfully. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Source dispatches integration to native implementations or private prepare/finish operations. Shared checks and input-freshness evidence remain mandatory between those operations. The private context uses validated opaque IDs; the Mercurial example prepares/resumes its native merge and revalidates parents before committing through native hooks. Writable adapter operations receive an owned temporary directory for hook exports, removed after the call. The isolated patch builder still uses Git internally. Full installed delivery acceptance remains required; P008 is pending.

## Verification

Eight focused Python integration cases pass for native/private Mercurial: failed gate recovery, rejected native commit recovery, mutated check inputs and conflict recovery. Three Rust protocol tests pass: malformed contexts never reach checks, failed gates never finish, and completion checks actual clean base state and removes temporary hook exports. The initial real-hook failures exposed read-only temporary storage and were repaired without bypassing checks. Structural lint and formatting pass. This VAC's full commit gate remains pending; full P007 acceptance remains unproven.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the private integration VAC through the staged gate, correcting failures. Audit remaining metadata-only/Git callers, including staging-index preparation, and test configuration update/rollback through private setup. Complete independent installed-consumer delivery acceptance for Git, Mercurial and the private extension. Release-template export belongs to the Git-only 0.2.0 to 0.3.0 migration. Verify all P007 acceptance before integration, then deliver P008. Preserve the installed development pin while testing candidates.
