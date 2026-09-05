# State

## Focus

Deliver P002: portable worker capabilities, standalone lint, separate configuration, declarative setup and shared update ownership. Apply complexity-discipline.

## Workspace

Branch: feature/worker-capabilities

Revision: 45dbdd5

The capability selection VAC is committed. The current VAC installs the worker's review resources, skill and consumer instructions.

## Progress

Init --review true installs the shared review configuration, contracts and prompts, plus the canonical review-project skill. Generated consumer instructions distinguish project sources from worker infrastructure. Just exposes review and lint. The existing manifest classifies review settings as configuration and prompts/skills/instructions as editable; release export includes enabled review assets.

## Verification

The capability selection VAC passed its full staged gate. Ten focused capability/package tests pass, including installed-binary configuration validation and MCP tool discovery without worker sources. The moved skill passes skill-creator validation. Strict lint has no errors. The current VAC's full staged gate is pending.

## Blockers

None. Setup, MCP registration, dependency checks and consumer reconfiguration/upgrade/rollback remain required. Reuse the existing package manifest and ownership rules.

## Next action

Commit the review resource installation VAC, then implement setup with standard assets, MCP registration and dependency diagnostics. Verify real external-consumer MCP review and the complete setup/update/rollback scenario before completing P002.
