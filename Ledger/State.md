# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: complete installed Mercurial/private delivery acceptance and repair its observed runtime-ignore defect.

## Workspace

Branch: feature/vcs-backends

Revision: fa6203b

P006 is integrated at 0c9cc39. Commit fa6203b verifies private upgrades; its full staged gate passed, including 562 Python tests. This VAC started from a clean feature/vcs-backends branch. No merge or rebase is pending; resume reports the previous gate completed successfully. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged. No test or commit process is currently running.

## Progress

P007 is active. Native/private Mercurial registration now appends the managed include to root .hgignore without replacing existing rules or duplicating the line on repeat setup. Existing hgrc registration remains preserved. Isolated reads still disable local commands/extensions and now see managed runtime exclusions. Independent consumers install the binary under a path with spaces and copy the external adapter locally; generated Just recipes and installed hooks complete feature delivery. Staging-index selection audit remains required; P008 is pending.

## Verification

Both installed-consumer cases pass in 154 seconds: bootstrap integration, feature creation, installed guarded commits, failed integration check, native recovery, exact merge parents, retained feature and next feature creation. Three existing setup cases pass, including preserved custom ignore rules and repeated registration. All 16 Rust registration/VCS tests pass, including isolation from repository commands. Structural lint passes. The original runtime/checks.json defect is repaired; the new consumer also declares ordinary Python-cache exclusions as the existing Git consumer does. This VAC's full commit gate remains pending.

## Blockers

No external blocker. Consumer temporary paths are not available across separate command invocations, so capture diagnostics within the test process. Keep read isolation, existing user ignore rules and clean-tree enforcement intact.

## Next action

Finish the installed-delivery/ignore repair VAC through the mandatory staged gate, correcting failures. Then audit metadata-only index callers without breaking staged configuration, complete P007 acceptance and integration, and deliver P008. Release-template export remains the Git-only 0.2.0 to 0.3.0 migration. Preserve the installed development pin while testing candidates.
