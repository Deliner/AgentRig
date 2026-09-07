# State

## Focus

Deliver P008 under the user's active goal. P006 and P007 are integrated. Current VAC: update selected-client model/API, hooks and MCP settings while preserving user preferences, repeat setup and rollback.

## Workspace

Branch: feature/agent-harnesses

Revision: 4d699ae

The branch started from master after P007 integration at 10f1d77. feature/vcs-backends is retained. Git confirms 4d699ae with the configuration-update VAC uncommitted and no merge or rebase in progress. The installed development pin is unchanged. User remarks remain in Ledger/Requests.md.

## Progress

Native custom-skill discovery and Claude's isolated user settings scope are committed. Current updates dispatch to the selected client, merge owned settings and retain unrelated preferences. Existing registration validation rejects disabled hooks before writes. Merged Codex user hooks use the existing manifest local-change receipt so repeated setup preserves them. Real Codex review and all three delegation modes passed in independent consumers after source removal; evidence is recorded in tooling/worker/examples/PORTABILITY.md. Real Claude review/delegate acceptance remains required.

## Verification

Commit 4d699ae passed the full gate: 599 Python and 145 Rust tests. Current configuration-update tests pass all 16 cases, including both clients, switching, disabled hooks, repeat setup and exact rollback. Mypy and architecture lint pass. Codex 0.153.4 completed real review run-pjyGqb and read/artifact/code runs run-53lZke, run-1MJbBH and run-PPoBtH with gpt-5.6-sol/high. Native Claude discovery/hook probes used synthetic credentials and do not prove a model response. The current VAC's full commit gate is pending.

## Blockers

Local Claude Code is 2.1.201. The user supplied admin@macbook.local, reachable through network-enabled just write SSH; its Claude is 2.1.263 at /Users/admin/.local/bin/claude. Its Keychain credential entry exists, but SSH previously could not read it (security exit 36, corresponding to interaction not allowed); launchctl asuser is denied. The user confirmed local login. A fresh SSH login-shell auth-status check still reports loggedIn=false. This does not establish absent local login. The prior request to unlock the login Keychain locally has no confirmed result; credentials have not been copied or exposed. This does not block implementation and deterministic tests.

## Next action

Commit the current configuration-update VAC through the mandatory gate and repair any failures. Complete independent real Claude review/delegate acceptance when authentication is usable, retaining the full P008 contract and development pin. Then record accepted results and integrate through feature-merge. Native discovery and deterministic adapter tests alone do not establish completion.
