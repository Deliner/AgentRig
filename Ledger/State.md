# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance is unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 0fe3289

Installed runtime 9aee4ae remains available. Hook/scaffold library ownership is committed. The current VAC separates shared installation receipt types and checksums from setup manifest generation, with an independent receipt contract and colocated tests. P009 acceptance and integration remain pending.

## Progress

Expanded inventories, repair guidance, four-language dependency analysis and measured Rust binding fixes are committed. Standard module declarations retain access checks without manufacturing use cycles. Calls, imports, type references and reexports retain cycle checks.

Delegation, environment, review, lint configuration/discovery, resource bundles, arguments and paths have cohesive owners and contracts. Static lint definitions remain independent of the sibling catalogue that composes language handlers. Contextual Git queries retain inherited environment and raw error semantics separately from the isolated VCS runner. Branch guards now belong with scaffold delivery.

Hook input helpers preserve util::object and util::text compatibility exports. Hook/scaffold references use crate paths inside the library; main imports the same services through agentrig. No duplicated module compilation or path attributes were introduced. Other util exports remain compatibility aliases.

Current receipt schema is independent of Config, Context and setup orchestration. Setup retains manifest generation and local-approval checks; update/reconciliation consumers import the shared types directly. Receipt checksums reuse the existing artifact digest implementation. D023's unchanged statement now links both the generator and schema owners.

## Verification

Commit b77e5fd passed its selective gate: 692 Python cases in 546.88 seconds plus Rust and other checks. Commit aeb8c5b passed its selective retry (95600 exited 0): 418 Python cases in 592.39 seconds plus all other configured checks. Its earlier unchanged review test timeout did not recur in the focused retry or final commit gate.

Commit 0fe3289 passed its selective gate (33146 exited 0): 418 Python cases in 535.60 seconds plus all configured checks. Its library boundary repair removed nine root-related cycles, leaving 14 cycles and 17 missing contracts.

Current receipt ownership compiles with cargo check --all-targets (40571). Three colocated schema tests pass: old receipts without local overrides roundtrip unchanged, local deletion differs from absence/replacement, and unknown receipt/entry fields remain rejected. Eleven native installation and update-application tests pass (88873, 29.00 seconds). Structural lint has no errors. Self-analysis in .tmp/p009-receipt-owner.json remains at 14 cycles and 17 missing contracts, with no new cycles or other findings; the new receipt contract passes. Commit verification remains pending.

## Blockers

No current blocker. Preserve behavior, permissions and maintained-source coverage.

## Next action

Commit receipt ownership through the affected gate. Then repair remaining scaffold cycles and missing contracts, using actual owners and consumers. Configuration loading currently calls upgrade recovery; recovery opens and validates the operation journal and saved plan, and update generation calls setup. Preserve journal checksum/project validation while separating shared state from orchestration; merely moving config.rs into config/mod.rs leaves those reverse dependencies. util.rs contains only compatibility reexports; preserve their public paths when completing the root inventory. Do not create arbitrary buckets or hide cycles through aliases.

The user requires affected commit tests plus a fast baseline, with the full suite only on merge. Repository catchall smoke selection prevents automatic full fallback, but current groups remain coarse and expensive. Refine selection against actual consumers; pytest targets must not overlap whole files/directories with contained nodes, because that can silently narrow collection. Required hooks remain authoritative.

Complete architecture coverage across maintained source/tests/docs/resources and justify service/generated/third-party exclusions. The Python probe .tmp/p009-python.yaml reports 69 unresolved local test imports and 12 missing contracts; assess explicit package imports before adding resolver modes. Discovery skips symlinks; canonical skills are under tooling/worker/assets/skills. Enable expanded checked policy, verify full P009 acceptance and integrate with feature-merge, retaining the branch.
