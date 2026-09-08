# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance is unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 546b6d4

Installed runtime 9aee4ae remains available. Settings, validated recovery-state ownership and the user's commit selection correction are committed. Pending changes move legacy memory parsing into history and command process execution into commands, with two architecture contracts and the D005 owner link. P009 acceptance and integration remain pending.

## Progress

Expanded inventories, repair guidance, four-language dependency analysis and measured Rust binding fixes are committed. Standard module declarations retain access checks without manufacturing use cycles. Calls, imports, type references and reexports retain cycle checks.

Delegation, environment, review, lint configuration/discovery, resource bundles, arguments and paths have cohesive owners and contracts. Static lint definitions remain independent of the sibling catalogue that composes language handlers. Contextual Git queries retain inherited environment and raw error semantics separately from the isolated VCS runner. Branch guards now belong with scaffold delivery.

Hook input helpers preserve util::object and util::text compatibility exports. Hook/scaffold references use crate paths inside the library; main imports the same services through agentrig. No duplicated module compilation or path attributes were introduced. Other util exports remain compatibility aliases.

Receipt schema is independent of Config, Context and setup orchestration. Setup retains manifest generation and local-approval checks; update/reconciliation consumers import the shared types directly. Receipt checksums reuse the existing artifact digest implementation. D023's unchanged statement links both generator and schema owners.

Current settings own static schema, strict validation and reading independently of recovery. config/mod.rs owns Context and explicit compatible exports. Recovery owns persisted update model types, metadata location and validated journal/plan reading; update execution calls that reader. Journal version, saved-plan checksum, project identity and completed-phase behavior are preserved. Hooks, setup and resume use the shared recovery owner. Three contracts cover settings, context loading and recovery; D005 follows its settings validation owner.

## Verification

Commit b77e5fd passed its selective gate: 692 Python cases in 546.88 seconds plus Rust and other checks. Commit aeb8c5b passed its selective retry (95600 exited 0): 418 Python cases in 592.39 seconds plus all other configured checks. Its earlier unchanged review test timeout did not recur in the focused retry or final commit gate.

Commit 0fe3289 passed its selective gate (33146 exited 0): 418 Python cases in 535.60 seconds plus all configured checks. Its library boundary repair removed nine root-related cycles, leaving 14 cycles and 17 missing contracts.

Commit ad73ab9 passed its selective gate (66752 exited 0): 418 Python cases in 503.48 seconds and all other configured checks. Receipt ownership left 14 cycles and 17 missing contracts.

Current settings/recovery split passes all-target compilation and Clippy (98679). Two colocated recovery tests reject changed plans, foreign projects and unknown journal versions, and verify finished phases. Forty-six native recovery, configuration update and hook cases pass (89461, 47.94 seconds). Structural lint has no errors. Self-analysis (.tmp/p009-project-recovery.json) removes six cycles without new cycles or other findings, leaving eight cycles and 16 missing contracts. Current commit verification remains pending.

## Blockers

Configuration access is restored: the user added commands.review-test.accepts_args and restored checks to the root configuration. config-check and structural lint pass. Command false defaults now have one owner in the runtime schema, and catchall smoke selection owns its previously duplicated target. The configuration has 500 nonblank lines; thresholds and test selection semantics are preserved.

The failed extraction exposed a real distinction: setup composes configuration packages, but settings::read decodes the live agentrig.yaml directly into Config. Root packages is unsupported. The unused tooling/checks.yaml extraction has been removed. Hook errors hid the schema cause behind the generic agentrig.yaml-required context and prevented both reads and repairs; preserve this recovery evidence.

## Next action

The user's check-selection correction is committed as 546b6d4; its required gate (28609) exited 0, including configured Rust and review checks. The settings/recovery commit is fe035eb; the earlier Verification paragraph describes its pre-commit checks. Commit the pending memory/commands changes, which passed 41 native cases in the preceding work session. The saved self-analysis .tmp/p009-memory-commands.json has six package/setup/update cycles and 15 missing contracts; broader P009 acceptance remains unfinished.

Current selection verification: eight existing staged/full/merge scenarios passed; six isolated probes using the repository maps verified memory, hooks, docs, Rust and review selection and unfiltered full checks. The configured Rust command accepts multiple libtest filters: scaffold:: plus hooks:: ran seven tests successfully and filtered out 85. Review changes retain the normal full crate command using the trailing -- separator; unrelated source changes skip review tests. These checks do not prove the pending commit gate or integration passed.

Commit the preserved source work through the required hook. Do not retry root package extraction. Then continue with the six package/setup/update cycles and missing architecture coverage. Current evidence identifies wizard orchestration, generated-file reads and installation storage as mixed owners; move full responsibilities and consumers rather than only renaming setup. The graph checks direct owners and crossed enclosing boundaries. Preserve util compatibility exports and actual consumers; do not hide cycles through aliases.

The user requires affected commit tests plus a fast baseline, with the full suite only on merge. Repository catchall smoke selection prevents automatic full fallback, but current groups remain coarse and expensive. Refine selection against actual consumers; pytest targets must not overlap whole files/directories with contained nodes, because that can silently narrow collection. Required hooks remain authoritative.

Complete architecture coverage across maintained source/tests/docs/resources and justify service/generated/third-party exclusions. The Python probe .tmp/p009-python.yaml reports 69 unresolved local test imports and 12 missing contracts; assess explicit package imports before adding resolver modes. Discovery skips symlinks; canonical skills are under tooling/worker/assets/skills. Enable expanded checked policy, verify full P009 acceptance and integrate with feature-merge, retaining the branch.
