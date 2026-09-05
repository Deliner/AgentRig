# State

## Focus

Deliver P002: portable worker capabilities, standalone lint, separate configuration, declarative setup and shared update ownership. Apply complexity-discipline.

## Workspace

Branch: feature/worker-capabilities

Revision: 36fde3b

The review frontend VAC is committed. The current VAC selects project capabilities and resolves review commands through worker.toml.

## Progress

Review and standalone lint retain their shared implementations. worker.toml now declares capabilities: lint defaults to enabled, and review is enabled by a config reference. The loader validates enabled review resources/contracts and rejects unknown capability keys or disabled lint with a configured lint check. Worker review config-check, mcp and run can use the project declaration while explicit-config calls remain available.

## Verification

The frontend VAC passed its full staged gate. Five focused capability/native-review tests pass, including configured MCP tools, disabled lint without a policy file, inconsistent gates, missing review resources and unknown capabilities. Strict lint has no errors. The current VAC's full staged gate is pending.

## Blockers

None. Resource installation, setup, and consumer reconfiguration/upgrade/rollback remain required. Reuse the existing package manifest and ownership rules.

## Next action

Commit the capability selection VAC, then implement setup with standard assets, MCP registration and dependency diagnostics. Verify real external-consumer MCP review and the complete setup/update/rollback scenario before completing P002.
