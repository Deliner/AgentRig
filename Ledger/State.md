# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance is unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: aeb8c5b

Installed runtime 9aee4ae remains available. Git query ownership and delivery guard consolidation are committed. The current VAC compiles hooks and scaffold in the library, moves optional JSON input helpers into hooks/input.rs and adds the hook directory contract. P009 acceptance and integration remain pending.

## Progress

Expanded inventories, repair guidance, four-language dependency analysis and measured Rust binding fixes are committed. Standard module declarations retain access checks without manufacturing use cycles. Calls, imports, type references and reexports retain cycle checks.

Delegation, environment, review, lint configuration/discovery, resource bundles, arguments and paths have cohesive owners and contracts. Static lint definitions remain independent of the sibling catalogue that composes language handlers. Contextual Git queries retain inherited environment and raw error semantics separately from the isolated VCS runner. Branch guards now belong with scaffold delivery.

Current hook input helpers preserve util::object and util::text compatibility exports. Hook/scaffold references now use crate paths inside the library; main imports the same services through agentrig. No duplicated module compilation or path attributes were introduced. Other util exports remain compatibility aliases.

## Verification

Commit b77e5fd passed its selective gate: 692 Python cases in 546.88 seconds plus Rust and other checks. Commit aeb8c5b passed its selective retry (95600 exited 0): 418 Python cases in 592.39 seconds plus all other configured checks. Its earlier unchanged review test timeout did not recur in the focused retry or final commit gate.

Current library ownership compiles with cargo check --all-targets (14845). Two colocated hook input tests pass, covering missing/malformed/non-object state and string/default field behavior. All 35 hook/reminder native cases pass (12592, 1.19 seconds). Structural lint has no errors.

Candidate self-analysis (90973, .tmp/p009-hook-services.json) removes nine root-related cycles without adding cycles or other findings, leaving 14 cycles and 17 missing contracts. Hook inventory and permissions pass. Current commit verification remains pending.

## Blockers

No current blocker. Preserve behavior, permissions and maintained-source coverage.

## Next action

Finish reviewing and commit hook service/library ownership through the affected gate. Then repair remaining scaffold cycles and missing contracts, using actual owners and consumers. util.rs contains only compatibility reexports; preserve their public paths when completing the root inventory. Do not create arbitrary buckets or hide cycles through aliases.

The user requires affected commit tests plus a fast baseline, with the full suite only on merge. Repository catchall smoke selection prevents automatic full fallback, but current groups remain coarse and expensive. Refine selection against actual consumers; pytest targets must not overlap whole files/directories with contained nodes, because that can silently narrow collection. Required hooks remain authoritative.

Complete architecture coverage across maintained source/tests/docs/resources and justify service/generated/third-party exclusions. The Python probe .tmp/p009-python.yaml reports 69 unresolved local test imports and 12 missing contracts; assess explicit package imports before adding resolver modes. Discovery skips symlinks; canonical skills are under tooling/worker/assets/skills. Enable expanded checked policy, verify full P009 acceptance and integrate with feature-merge, retaining the branch.
