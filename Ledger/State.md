# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: make staging capabilities explicit and route saved index evidence through the selected backend.

## Workspace

Branch: feature/vcs-backends

Revision: 9d98a07

P006 is integrated at 0c9cc39. Commit 9d98a07 repairs installed Mercurial ignores and verifies complete installed native/private delivery; its full gate passed, including 564 Python tests. This VAC started from a clean feature/vcs-backends branch. No merge or rebase is pending; resume reports the previous gate completed successfully. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 is active. Source now owns selected index reads and explicit private-index capability errors. Gate preparation retains native index fingerprint/export before staged configuration is available, then validates the staged backend before adapter execution or checks. Evidence finish/resume use the configured backend instead of metadata discovery. Native preparation is explicitly named and documented; invalid unstaged YAML cannot replace staged settings. P008 remains pending.

## Verification

Nine focused staging/evidence tests pass, including invalid unstaged Git configuration, staged private capability rejection before adapter launch, changed-backend evidence without native fallback, Mercurial unsupported staging, and existing revision/content/index freshness behavior. Structural lint, formatting and diff whitespace pass. This VAC's full commit gate remains pending. Full P007 completion still requires final acceptance reconciliation and branch integration.

## Blockers

No external blocker. Consumer temporary paths are not available across separate command invocations, so capture diagnostics within the test process. Keep source selection, read isolation, existing user ignore rules and clean-tree enforcement intact.

## Next action

Commit the staging capability VAC through the mandatory gate, correcting failures. Reconcile every P007 acceptance item against current implementation, docs and actual native/private consumer checks; update Plan only when complete, then integrate through feature-merge and proceed to P008. Native index bootstrap, the isolated patch builder's internal Git workspace and the Git-only 0.2.0 to 0.3.0 release-template export are explicit boundaries, not private source fallback. Preserve the installed development pin while testing candidates.
