# State

## Focus

Deliver P002: native worker capabilities, standalone lint, separate configuration, declarative setup and shared update ownership. Apply complexity-discipline.

## Workspace

Branch: feature/worker-capabilities

Revision: a84e9d0

The review integration VAC is committed after its full gate. The current VAC adds a native standalone lint executable over the worker's shared library.

## Progress

Review is embedded in the worker and retains one implementation under tooling/worker/review. Lint, diagnostics and common utilities now form the worker library used by both executables. The release build produces discipline-worker and discipline-lint. Optional lint skill_root resolves a central skill bundle relative to the policy file while selectors still address the assessed project; existing configurations retain project-relative skill validation. Both successful findings and configuration errors point to the external guidance correctly.

## Verification

Three new portable-lint tests pass: worker/standalone parity for Git and non-Git consumers, no consumer file changes, and actionable rejection of an incompatible language. Strict lint passes without errors. The prior native-review VAC passed all worker/review checks. The full staged gate for this standalone-lint VAC remains pending.

## Blockers

None. Project capabilities, review frontend declaration, resource installation, setup and consumer reconfiguration/upgrade/rollback remain required. Reuse the existing package manifest and ownership rules.

## Next action

Commit the standalone-lint VAC. Then implement worker capability configuration and setup from the user's declaration, including standard assets, MCP registration and dependency diagnostics. Verify real external-consumer MCP review and the complete setup/update/rollback scenario before completing P002.
