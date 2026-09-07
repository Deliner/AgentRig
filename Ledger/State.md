# State

## Focus

Integrate accepted P007, then deliver P008 under the user's active implementation goal; P006 is integrated. Current VAC: repair cached review tests that retained paths into a deleted exported source tree.

## Workspace

Branch: feature/vcs-backends

Revision: 45a6866

P006 is integrated at 0c9cc39. P007 completion commit 45a6866 passed its full gate with 567 Python and 138 Rust tests. Its integration failed in review-test after all 567 Python tests passed. No merge or rebase is pending; the feature branch is retained. Resume reports completed focused review-test evidence. User remarks remain in Ledger/Requests.md. The installed development pin is unchanged.

## Progress

P007 implementation acceptance is recorded in Plan/007.md, but base integration remains unfinished. Three review test fixtures now resolve CARGO_MANIFEST_DIR at test execution instead of embedding a temporary compilation path. Review/repair assertions include the failed report. Product behavior and verification thresholds are unchanged. P008 remains pending; preliminary inspection found Codex-specific setup and review/delegate launch owners.

## Verification

The failed review stage reproduced missing external_vcs.py under the deleted exported tree /tmp/.tmpKOhDOo. Recompiling execution tests removed their failure while cached external_vcs tests still failed with that exact path. After correcting all three owners, just check --only review-test passed all 64 Rust review tests. The correction is rustfmt formatted; its full commit gate and the integration retry remain pending.

## Blockers

P007 has no unresolved implementation blocker; commit and integration gates remain required. For later P008 real-client acceptance, local Codex is 0.153.4 and Claude Code is 2.1.201. The user supplied admin@macbook.local, reachable through network-enabled just write SSH; its Claude is 2.1.263 at /Users/admin/.local/bin/claude. Its Keychain credential entry exists, but SSH cannot read it (security exit 36, corresponding to interaction not allowed); launchctl asuser is denied. The user was asked to unlock the login Keychain locally. SSH loggedIn=false does not establish absent local login; credentials have not been copied or exposed. This does not block P008 implementation and deterministic tests.

## Next action

Commit this focused test repair through the mandatory full gate, then retry just feature-merge and verify integration into master. Keep the feature branch. Start P008 on a new feature branch only after integration, activate its contract under the existing goal, and deliver actual Codex/Claude setup, review and delegation with real-client acceptance. Preserve the installed development pin while testing candidates. Native index bootstrap, the internal Git patch builder and the Git-only release migration remain explicit supported boundaries.
