# State

## Focus

Deliver P002: portable worker capabilities, standalone lint, separate configuration, declarative setup and shared update ownership. Apply complexity-discipline.

## Workspace

Branch: feature/worker-capabilities

Revision: dba2df9

The native review and standalone lint VACs are committed. The current VAC makes the review executor explicit and rejects unsupported frontends.

## Progress

Review has one implementation embedded in worker. Standalone discipline-lint shares the worker engine and supports external policy guidance without installing files in the assessed project. Reviewers now declare frontend; Codex is the implemented executor and the compatibility default. Configuration validation rejects other frontends before execution; resolved configuration records the executor for reports.

## Verification

The standalone lint commit passed its full staged gate. Three focused review configuration tests pass, including explicit/default Codex, report configuration serialization and unsupported-executor diagnostics. Strict lint has no errors. The current VAC's full staged gate is pending.

## Blockers

None. Project capability selection, resource installation, setup, and consumer reconfiguration/upgrade/rollback remain required. Reuse the existing package manifest and ownership rules.

## Next action

Commit the explicit frontend VAC, then implement capability configuration and setup with standard assets, MCP registration and dependency diagnostics. Verify real external-consumer MCP review and the complete setup/update/rollback scenario before completing P002.
