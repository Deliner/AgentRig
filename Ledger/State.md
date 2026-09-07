# State

## Focus

Deliver P008 under the user's active goal. P006 and P007 are integrated. Current VAC: explicit project-client selection with Claude setup, preview, hooks/MCP registration and selected executor diagnostics.

## Workspace

Branch: feature/agent-harnesses

Revision: b6d7a96

The branch started from master after P007 integration at 10f1d77. feature/vcs-backends is retained. Resume confirmed a clean b6d7a96 with no merge or rebase before this VAC. Review is committed at 94ce7f1 and delegation at b6d7a96. P006 is integrated at 0c9cc39. The installed development pin is unchanged. User remarks remain in Ledger/Requests.md.

## Progress

Review and all delegation modes support Claude. The current uncommitted VAC adds top-level project frontend selection and init/wizard choice. Claude uses .claude/settings.json, .mcp.json and CLAUDE.md importing shared AGENTS.md. Existing reconciliation preserves settings, permissions and unrelated hooks/MCP; managed conflicts fail before writing. Preview omits unrelated MCP data. Doctor and preview share selected review/delegate executable discovery. Codex MCP registrations forward declared Claude credential references. The shared Frontend enum moved from delegation into environment. Project model/API mapping, native skill discovery and real-client acceptance remain required.

## Verification

Delegation b6d7a96 passed its full commit gate: 577 Python and 145 Rust tests. Current setup work passed 88 setup/capability/review tests before the final wizard and preview adjustments, then 13 wizard cases and five Claude preview cases. Standalone tests exposed an import-order issue; setup assertions now stay with scaffold helpers and client protocol fixtures stay in test_review.py. No import-path configuration was added. The final capability/review run passed 25 tests. Mypy, structural lint, whitespace checks and Rust compilation pass after repair. The full commit gate is pending. Direct registered hook/MCP tests prove adapter execution, not real client discovery or model acceptance.

## Blockers

For P008 real-client acceptance, local Codex is 0.153.4 and Claude Code is 2.1.201. The user supplied admin@macbook.local, reachable through network-enabled just write SSH; its Claude is 2.1.263 at /Users/admin/.local/bin/claude. Its Keychain credential entry exists, but SSH cannot read it (security exit 36, corresponding to interaction not allowed); launchctl asuser is denied. The user was asked to unlock the login Keychain locally. SSH loggedIn=false does not establish absent local login; credentials have not been copied or exposed. This does not block P008 implementation and deterministic tests.

## Next action

Commit the current project setup VAC through the mandatory gate and repair any failures. Then complete shared project model/API settings and native skill discovery, verify composition/update behavior and actual client hook semantics, and run independent real-client acceptance for both clients before integrating P008. Preserve the development pin and the full P008 contract; current adapter tests alone do not establish completion.
