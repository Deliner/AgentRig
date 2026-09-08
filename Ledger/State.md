# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance and integration remain unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: cdc9c17

Installed runtime 9aee4ae remains pinned. Current unfinished VAC moves the existing upgrade test package into tooling/worker/src/scaffold/upgrade/tests. The predecessor fixture retains the fixed revision and WORKER_SOURCE_ROOT override; relative repository/example paths, D024 application link and configured command/check targets follow the moves. No commit gate has started for this VAC yet.

## Progress

Expanded exact inventories, repair guidance, four-language dependency analysis and Rust binding fixes are committed. Literal file-level Rust module paths are supported with explicit unsupported-form diagnostics and compiler parity tests.

Review tests now belong to execution, run, MCP, configuration, snapshot and VCS. Repository fixture builders stay with VCS; common review fixtures live in review/testing. Reverse VCS-to-config/snapshot and run-to-MCP test dependencies were removed.

All lint Python tests now live beside architecture, config, languages, inventory or composed CLI behavior. Shared consumer setup lives in lint/testing. Common pytest worker selection lives in tooling/conftest.py and preserves WORKER_BINARY behavior. Exact oracle paths and decision application links follow the original test bodies. No external exposure of lint test fixtures remains after their consumers moved into lint.

Current delegate moves preserve scenario bodies: test_delegate_run.py and test_delegate_mcp.py under run/tests; test_delegate_code.py and test_delegate_vcs.py under code/tests; test_delegate_config.py under config/tests. Shared helpers extracted from old run/MCP test modules now live in delegate/testing/consumer.py and mcp.py. Imports, example adapter path, command inputs and affected test targets follow the moves. Exact contracts describe all four new directories.

P010 is separately planned and pending for malformed-configuration hook deadlock. Do not implement it instead of P009 or touch voxel-rust.

## Verification

Delegate ownership commit cdc9c17 passed gate 32828: 94 Rust tests, 217 selected Python tests in 127.08 seconds, all review tests and other configured checks.

Current upgrade relocation passes 34 tests in 51.57 seconds (91276 terminal 0), mypy on 54 sources, Ruff, formatting, memory check 4281 and git diff --check. Combined probe has no new dependency findings; ten older missing contracts and the two example Rust root findings remain.

Latest committed VAC 9c645e2 passed normal gate 72033: 94 Rust tests, 253 selected Python tests, review tests and all configured checks. Prior ca3d4eb passed gate 88878: 94 Rust tests, 253 Python tests in 29.84 seconds, review tests and all checks. Review ownership a371fd0 passed retry 71837: 73 Rust tests and 694 Python tests in 525.07 seconds plus review and other checks.

Current delegation relocation passes 73 tests in 38.09 seconds (89216 terminal 0), mypy across 54 sources, Ruff, formatting, memory check 35550, structural lint and git diff --check. No commit or integration proof yet for these changes.

Combined Rust/Python probe .tmp/p009-combined.yaml reports no new access/cycle/unresolved import findings. Remaining findings are 11 older missing contracts under tooling/tests and examples plus two incomplete-root findings for the Rust example. This probe covers source extensions and is not full maintained-tree acceptance. Main checked tooling/worker/lint.yaml still does not enable architecture.

## Blockers

No operational blocker. Preserve behavior, exact oracles, permissions and full maintained-tree scope. Root agentrig.yaml is strict; do not extract unsupported configuration packages or update the installed pin as a workaround.

## Next action

Inspect and stage the upgrade test ownership VAC, commit through the normal hook and resolve actual findings. Keep the current VAC isolated until accepted. Resume a live gate by its process handle; do not restart a quiet process. Delegate and lint test ownership VACs are already accepted.

Then colocate remaining native Python tests: scaffold behavior with commands/gate/hooks/memory/package/setup owners, upgrades with upgrade, launcher with distribution and review CLI consumers with review. Shared helpers must remain with actual owners. Update configured check targets, invariant oracles, decision application links, fixture discovery and __file__ paths; read matching memory skills first. Existing scaffold helpers have actual feedback/gate/Git cycles that need ownership repair, not exclusion.

Broaden architecture to maintained tests, docs and resources with explicit justified service/generated/third-party exclusions and exact descriptions. Configure the example Rust crate root and all maintained language roots; canonical skills live under tooling/worker/assets/skills and symlinks are skipped. Enable architecture in checked lint.yaml, verify both binaries and all four languages, complete full P009 acceptance and integrate through feature-merge while retaining the branch. A clean subset probe alone is not completion.
