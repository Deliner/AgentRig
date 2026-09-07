# State

## Focus

Deliver P008 under the user's active goal. P006 and P007 are integrated. Current VAC: selected native discovery paths for installed custom project skills.

## Workspace

Branch: feature/agent-harnesses

Revision: 5979f0e

The branch started from master after P007 integration at 10f1d77. feature/vcs-backends is retained. Git confirmed clean 5979f0e before this VAC after the model/API commit gate completed. Review is committed at 94ce7f1, delegation at b6d7a96 and project registrations at 5173a10. P006 is integrated at 0c9cc39. The installed development pin is unchanged. User remarks remain in Ledger/Requests.md.

## Progress

Review, all delegation modes, project registrations and model/API references support Claude. Current uncommitted work places custom project skills under .agents/skills for Codex and .claude/skills for Claude, using the existing resource copier and manifest. The shared environment test now exercises both clients, including actual registered hooks/MCP and preservation of local skill support edits. Shipped worker guidance retains paths.skills and explicit hook/instruction routing; no second instruction store or symlink lifecycle was added. Composition/update acceptance for Claude and real review/delegate model runs remain required.

## Verification

Model/API 5979f0e passed the full gate: 598 Python and 145 Rust tests. Current skill placement passed seven focused environment cases, then all 24 capability tests; mypy, structural lint and whitespace checks pass. Native disposable-consumer probes confirmed discovery: Codex 0.153.4 app-server skills/list returned the installed skill with no errors; Claude Code 2.1.201 emitted it in init.skills and slash_commands and executed a startup hook. Both used isolated client homes. Claude used a synthetic key and an unreachable loopback API, timed out after discovery and did not complete a model request. These probes prove discovery, not successful model execution. The current full commit gate is pending.

## Blockers

For P008 real-client acceptance, local Codex is 0.153.4 and Claude Code is 2.1.201. The user supplied admin@macbook.local, reachable through network-enabled just write SSH; its Claude is 2.1.263 at /Users/admin/.local/bin/claude. Its Keychain credential entry exists, but SSH cannot read it (security exit 36, corresponding to interaction not allowed); launchctl asuser is denied. The user was asked to unlock the login Keychain locally. SSH loggedIn=false does not establish absent local login; credentials have not been copied or exposed. This does not block P008 implementation and deterministic tests.

## Next action

Commit the current skill-placement VAC through the mandatory gate and repair any failures. Then verify Claude composition/update behavior, native delegate skill loading and actual hook/MCP semantics, and run independent real review/delegate acceptance for both clients before integrating P008. Preserve the development pin and the full P008 contract; discovery and adapter tests alone do not establish completion.
