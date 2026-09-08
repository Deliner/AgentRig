# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance is unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: ad73ab9

Installed runtime 9aee4ae remains available. Shared receipt ownership is committed. The current VAC separates project settings, validated recovery-state reading and context loading while preserving config::Context and configuration exports. P009 acceptance and integration remain pending.

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

No current blocker. Preserve behavior, permissions and maintained-source coverage.

## Next action

Commit settings/recovery ownership through the affected gate. Then repair the eight remaining cycles in scaffold memory, package/setup and update generation and add missing contracts. The graph checks both direct owners and crossed enclosing boundaries; nesting shared state under update execution would restore the cycle. util.rs contains only compatibility reexports; preserve their public paths when completing the root inventory. Do not create arbitrary buckets or hide cycles through aliases.

The user requires affected commit tests plus a fast baseline, with the full suite only on merge. Repository catchall smoke selection prevents automatic full fallback, but current groups remain coarse and expensive. Refine selection against actual consumers; pytest targets must not overlap whole files/directories with contained nodes, because that can silently narrow collection. Required hooks remain authoritative.

Complete architecture coverage across maintained source/tests/docs/resources and justify service/generated/third-party exclusions. The Python probe .tmp/p009-python.yaml reports 69 unresolved local test imports and 12 missing contracts; assess explicit package imports before adding resolver modes. Discovery skips symlinks; canonical skills are under tooling/worker/assets/skills. Enable expanded checked policy, verify full P009 acceptance and integrate with feature-merge, retaining the branch.
