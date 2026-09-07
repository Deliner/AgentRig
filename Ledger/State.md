# State

## Focus

Deliver P008 under the user's active goal. P006 and P007 are integrated. Current VAC: enable native Claude delegate skill discovery through its isolated user settings scope.

## Workspace

Branch: feature/agent-harnesses

Revision: 23db8dc

The branch started from master after P007 integration at 10f1d77. feature/vcs-backends is retained. Git confirmed clean 23db8dc before this VAC after the project-skill commit gate completed. Model/API settings are committed at 5979f0e, review at 94ce7f1, delegation at b6d7a96 and project registrations at 5173a10. P006 is integrated at 0c9cc39. The installed development pin is unchanged. User remarks remain in Ledger/Requests.md.

## Progress

Project custom skills now use each client's native discovery path. Native probes exposed a separate delegate issue: empty --setting-sources hides skills under its private CLAUDE_CONFIG_DIR. Current uncommitted work selects user instead; the private home and explicit settings/MCP remain unchanged. Native comparison confirmed discovery, single startup-hook execution and exclusion of a project settings hook. Evidence and limits are recorded in tooling/worker/examples/PORTABILITY.md. Configuration-update code still unconditionally reads .codex/config.toml and merges only MCP fields; Claude updates and model/API update behavior need repair and acceptance. Real review/delegate model runs remain required.

## Verification

Project skills 23db8dc passed the full gate: 599 Python and 145 Rust tests. The current delegate scope correction passed the existing Rust frozen-skill/hook/MCP sandbox test and eight Claude delegation/configuration cases. Native Claude Code 2.1.201 discovered a private-home skill with user scope; the private startup hook ran once and the project hook zero times. These probes used synthetic credentials and an unreachable loopback API, so they do not prove a successful model request. The current full commit gate is pending.

## Blockers

For P008 real-client acceptance, local Codex is 0.153.4 and Claude Code is 2.1.201. The user supplied admin@macbook.local, reachable through network-enabled just write SSH; its Claude is 2.1.263 at /Users/admin/.local/bin/claude. Its Keychain credential entry exists, but SSH cannot read it (security exit 36, corresponding to interaction not allowed); launchctl asuser is denied. The user was asked to unlock the login Keychain locally. SSH loggedIn=false does not establish absent local login; credentials have not been copied or exposed. This does not block P008 implementation and deterministic tests.

## Next action

Commit the current delegate scope VAC through the mandatory gate and repair any failures. Then repair selected-client configuration updates with preservation and rollback tests, verify actual client hook/MCP semantics, and run independent real review/delegate acceptance for both clients before integrating P008. Preserve the development pin and the full P008 contract; discovery and adapter tests alone do not establish completion.
