# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance is unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: ce19b5e

Current VAC moves the lint diagnostic result and text rendering to diagnostics/lint.rs beside the existing shared guidance owner in diagnostics/mod.rs. Architecture evaluation consumes the result directly; lint::Diagnostic remains a compatibility reexport. The D022 application link follows its marked owner. Development pin is unchanged. Integration remains pending.

## Progress

Inventory, repair guidance, Rust binding analysis, delegation/environment/review ownership and declaration/use cycle correction are committed. Standard Rust module declarations remain subject to access checks but no longer manufacture use cycles. Calls, imports, type references and reexports retain cycle checks.

Shared artifact bytes and revision snapshot ownership are committed in ce19b5e. The current diagnostic move removes one reverse dependency on lint orchestration while preserving JSON fields, text formatting and failure codes. Its exact inventory and public boundary are recorded in diagnostics/architecture.yaml. Other edges still produce cycles in the same directories; this move does not claim those cycles are resolved.

## Verification

Commit ce19b5e passed all gates (75682 exited 0): 676 Python tests in 648.01 seconds and 147 Rust tests.

Current VAC passed 95 lint/configuration/explain/recovery CLI tests in 196.57 seconds (50999), structural lint and whitespace checks. Rust sources are formatted. The first focused invocation used a nonexistent explain test path and ran no tests; the corrected invocation passed. Source probe (34881 exited 0) still reports 34 cycles and 28 missing contracts, with no other findings. This probe covers Rust sources, not the complete maintained repository. Mandatory commit gate remains required.

## Blockers

No current blocker. Preserve permissions and maintained-source coverage.

## Next action

Commit this diagnostic ownership VAC through the mandatory gate. Continue remaining worker lint/scaffold ownership and measured dependency repairs. Complete inventory and dependency checks across maintained source/tests/docs/resources, justify service/generated/third-party exclusions, and fix VCS empty-directory coverage without bypassing ignore semantics: current VCS inventory derives directories only from working files. Existing discovery skips symlinks; canonical skills live under tooling/worker/assets/skills rather than the .agents/skills alias. Enable expanded checked policy, verify full P009 acceptance and integrate with feature-merge, retaining the branch.
