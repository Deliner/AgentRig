# State

## Focus

Deliver P008 under the user's active goal. P006 and P007 are integrated. Current VAC: selected-client isolated review for Codex and Claude Code with shared validation and failure handling.

## Workspace

Branch: feature/agent-harnesses

Revision: 10f1d77

The branch started clean from master after P007 integration. feature/vcs-backends is retained. Resume confirms no merge or rebase and completed baseline checks. P006 is integrated at 0c9cc39. The installed development pin is unchanged. User remarks remain in Ledger/Requests.md.

## Progress

P007 repair fd49e05 and integration 10f1d77 are complete. Plan activates P008. The first VAC adds per-role Codex/Claude Code review selection, Claude print-mode launch with explicit read-only Stop settings, shared parent-owned validation and external credential references. Credentials and environment-reference helpers now have one shared owner used by review and delegation. New role reports identify the client. Existing Codex auth-file fallback remains; explicit API references work without it. Project setup and delegation still select only Codex and remain required P008 work.

## Verification

P007's commit and integration gates each passed 567 Python and 138 Rust tests. For this VAC, the full review-test stage passed 67 tests before adding two credential cases; the final five-test harness suite then passed. All 38 delegate configuration tests, structural lint and diff whitespace checks pass. Rust and Python changes are formatted. These client-boundary tests use deterministic executors in real bubblewrap; they verify mixed selection, correction, exhaustion, timeout, missing references and source preservation, not real model acceptance. The full commit gate is pending.

## Blockers

For P008 real-client acceptance, local Codex is 0.153.4 and Claude Code is 2.1.201. The user supplied admin@macbook.local, reachable through network-enabled just write SSH; its Claude is 2.1.263 at /Users/admin/.local/bin/claude. Its Keychain credential entry exists, but SSH cannot read it (security exit 36, corresponding to interaction not allowed); launchctl asuser is denied. The user was asked to unlock the login Keychain locally. SSH loggedIn=false does not establish absent local login; credentials have not been copied or exposed. This does not block P008 implementation and deterministic tests.

## Next action

Finish the current selected-client review VAC through the mandatory commit gate and repair any failures. Then complete selected-client project setup and read/artifact/code delegation with shared environment declarations, skills/hooks/MCP and supported model/API settings. Verify independent real-client acceptance for both clients and integrate P008. Preserve the development pin and the full P008 contract; the current review tests alone do not establish completion.
