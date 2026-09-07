# State

## Focus

Deliver P008 under the user's active goal. P006 and P007 are integrated. Current VAC: record real Claude/OpenRouter acceptance and the remaining code-mode provider limit.

## Workspace

Branch: feature/agent-harnesses

Revision: 81caa2d

The branch started from master after P007 integration at 10f1d77. feature/vcs-backends is retained. Git confirmed clean 81caa2d after the configuration-update commit and before these evidence edits. No merge or rebase is in progress. The installed development pin is unchanged. User remarks remain in Ledger/Requests.md.

## Progress

Selected-client setup, model/API updates, native skill discovery, review and delegation implementations are committed. Codex real review and all three delegation modes passed. The user authorized local Claude through existing OpenRouter credentials, using a cheapest or free model. Local Claude 2.1.201 works through that route. Free Nemotron completed real review, read and artifacts acceptance after source removal. Code acceptance remains incomplete. Evidence is recorded in tooling/worker/examples/PORTABILITY.md. All launched probes are terminal; no model run or gate is currently running.

## Verification

Commit 81caa2d passed the full gate: 607 Python and 145 Rust tests, including 16 configuration-update cases. Claude review run-o7px9A returned validated PASS and its runtime directory was removed. Read run-rSxFAL matched independent skill/hook/service values; artifacts run-dsgPOK also retained an observed MCP tools/call, startup output and unchanged checkout. Their private directories were removed. All used nvidia/nemotron-3-super-120b-a12b:free. Earlier Claude 3 Haiku connected but failed the full task contracts. Successful Claude code delegation is not established.

## Blockers

OpenRouter now returns HTTP 429, free-models-per-day, observed in focused code run-FPUmYi. Earlier combined code probes returned HTTP 404. Free inference cannot continue until the quota changes. Actual spending: $0.13806775 for these probes, $1.41928505 remaining; Claude's gateway cost estimates are not actual billing. An optional question about up to $0.30 for a stronger Claude is pending without approval; the assistant committed to free-only continuation meanwhile. The Mac local login is acknowledged, but SSH login-shell and allocated-terminal checks still fail to access its Keychain entry. OpenRouter provides a working alternative without copying Mac credentials.

## Next action

Commit this evidence VAC through the normal gate. Resume .tmp/agentrig-claude-free-code.py after free capacity resets, or use a user-approved alternative budget/authentication route. Inspect the existing run before retrying; preserve the full P008 acceptance contract and development pin. Only after code acceptance passes, record completion and integrate with feature-merge. P008 and the goal remain incomplete.
