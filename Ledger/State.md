# State

## Focus

Deliver P008 under the user's active goal. P006 and P007 are integrated. Current VAC: Claude Code read/artifact/code delegation using shared isolation, task validation and jobs.

## Workspace

Branch: feature/agent-harnesses

Revision: 94ce7f1

The branch started from master after P007 integration at 10f1d77. feature/vcs-backends is retained. Resume confirms no merge or rebase. The selected-client review VAC is committed at 94ce7f1; the current delegation changes are uncommitted. P006 is integrated at 0c9cc39. The installed development pin is unchanged. User remarks remain in Ledger/Requests.md.

## Progress

Selected-client review is committed. The current VAC adds Claude delegation with explicit settings, frozen skills/programs/hooks/MCP resources and external credential references. Claude structured_output feeds the existing parent-owned schema, artifact and code verification. The parent reserves result.json without following model-created paths. Jobs retain shared cancellation, timeout, reports and cleanup. Existing Codex execution remains covered. Project setup still selects only Codex and remains required P008 work.

## Verification

The review VAC's full commit gate passed 567 Python and 143 Rust tests. After recovering interrupted output and confirming no test processes remained, the current delegation VAC passed 62 Python tests across run/code/config and 13 Rust delegate tests. Structural lint has no errors. Tests cover both clients, all modes, invalid responses, timeout, cancellation, frozen resources, read-only inputs/configuration and code check failure. Claude fixtures are deterministic executors in real bubblewrap, not real model acceptance. The current full commit gate is pending.

## Blockers

For P008 real-client acceptance, local Codex is 0.153.4 and Claude Code is 2.1.201. The user supplied admin@macbook.local, reachable through network-enabled just write SSH; its Claude is 2.1.263 at /Users/admin/.local/bin/claude. Its Keychain credential entry exists, but SSH cannot read it (security exit 36, corresponding to interaction not allowed); launchctl asuser is denied. The user was asked to unlock the login Keychain locally. SSH loggedIn=false does not establish absent local login; credentials have not been copied or exposed. This does not block P008 implementation and deterministic tests.

## Next action

Inspect and commit the current delegation VAC through the mandatory gate, repairing any failures. Then complete selected-client project setup with shared environment declarations, skills/hooks/MCP and supported model/API settings. Verify independent real-client acceptance for both clients and integrate P008. Preserve the development pin and the full P008 contract; deterministic client tests alone do not establish completion.
