# State

## Focus

Deliver P008 under the user's active goal. P006 and P007 are integrated. Current VAC: project model, reasoning effort and external API-key references mapped to native Codex and Claude settings.

## Workspace

Branch: feature/agent-harnesses

Revision: 5173a10

The branch started from master after P007 integration at 10f1d77. feature/vcs-backends is retained. Resume confirmed clean 5173a10 with no merge or rebase before this VAC; its previous gate is completed. Review is committed at 94ce7f1, delegation at b6d7a96 and project registrations at 5173a10. P006 is integrated at 0c9cc39. The installed development pin is unchanged. User remarks remain in Ledger/Requests.md.

## Progress

Review, all delegation modes and project registrations support Claude. Current uncommitted work adds optional agent.model, reasoning_effort and api key_env/base_url declarations. Codex uses a named Responses API provider; Claude uses native apiKeyHelper plus endpoint settings. Setup and preview retain references without reading secrets, preserve conflicting native settings and reject unsupported declarations. Omitting agent retains existing behavior; omitting api retains native authentication. No new client launcher or credential storage was introduced. Native skill discovery, composition/update acceptance and real-client execution remain required.

## Verification

Project registration 5173a10 passed its full commit gate: 584 Python and 145 Rust tests. Current model/API work passed all 43 package and strict-policy tests, including 14 new cases for both clients, endpoint defaults, references, conflicts and invalid settings. The generated Claude helper was executed with a synthetic shell-sensitive key and with a missing reference. Rust compilation, mypy, structural lint and whitespace checks pass after repair. The full commit gate is pending. Configuration and adapter tests do not prove real client discovery or model acceptance.

## Blockers

For P008 real-client acceptance, local Codex is 0.153.4 and Claude Code is 2.1.201. The user supplied admin@macbook.local, reachable through network-enabled just write SSH; its Claude is 2.1.263 at /Users/admin/.local/bin/claude. Its Keychain credential entry exists, but SSH cannot read it (security exit 36, corresponding to interaction not allowed); launchctl asuser is denied. The user was asked to unlock the login Keychain locally. SSH loggedIn=false does not establish absent local login; credentials have not been copied or exposed. This does not block P008 implementation and deterministic tests.

## Next action

Commit the current model/API VAC through the mandatory gate and repair any failures. Then complete native skill discovery, verify composition/update behavior and actual client hook semantics, and run independent real-client acceptance for both clients before integrating P008. Preserve the development pin and the full P008 contract; current adapter tests alone do not establish completion.
