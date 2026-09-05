# State

## Focus

Implement P001, the user's configurable isolated review MCP and shared review-runner. Follow the detailed acceptance in Plan/001.md and apply complexity-discipline throughout.

## Workspace

Branch: feature/review-mcp

Revision: bde2c65

The prior upgrade goal is integrated on master. This dedicated branch starts the resumed product feature. Product implementation belongs under Project/review-mcp.

## Progress

Read the current skills and project contracts. Updated P001 with the user's configurable tools/reviewers, machine contract, snapshot scope, isolated Codex, common validator, deterministic report, repeated-review and cleanup requirements. Installed Codex is 0.153.4 and advertises ignore-user-config, ignore-rules, ephemeral mode and explicit hook trust for vetted automation. A real isolated gpt-5.6-luna invocation confirmed blocking Stop continuation: initial final response, hook-requested correction, valid file and successful exit. Both codex and codex-code-mode-host are required mounts.

## Verification

Six Rust tests now pass for configuration-relative resources, invalid configuration/assignments, response schema and identity, missing/duplicate IDs, FAIL/N/A consistency, and missing/oversized/symlink outputs. Strict lint and Clippy pass. Product Cargo test/Clippy/rustfmt commands are registered in worker.toml and Project participates in source scope. The validator/configuration VAC passed its full staged gate: 208 existing tests plus six product tests. Three additional real-Git snapshot tests now pass: committed content differs from live/untracked files, deletions/renames cannot cross visibility, and secret/control paths or symlinks cannot be exposed. The snapshot VAC remains uncommitted.


Codex 0.153.4 with a fresh auth-only home and ignored user configuration ran inside bubblewrap. The Stop hook rejected a missing response, and the model continued and wrote the requested exact JSON. Earlier timeouts were diagnosed as the missing codex-code-mode-host executable. Copied authentication and temporary smoke directories were removed after recording evidence in Project/review-mcp/COMPATIBILITY.md. Production isolation, validator, protected counter and MCP-client acceptance are still unverified.

## Blockers

None. Use the supported Stop hook; no runner-only fallback is needed for this installed version. Keep the attempt counter in the parent process and expose only validation requests through a per-reviewer endpoint, since a hook executing inside the sandbox cannot safely own a critic-writable counter file.

## Next action

Commit the snapshot VAC, then implement isolated reviewer execution with protected attempts, deterministic persistent reports/cleanup, and configured MCP tools plus CLI/Just/skill. Verify a real client and complete all acceptance before integration.
