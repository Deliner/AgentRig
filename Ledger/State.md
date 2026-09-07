# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance is unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 3a1967f

Runtime promotion and affected-test policy are committed; installed runtime 9aee4ae is available. Extraction ownership is committed in 3a1967f. The current P009 VAC separates rule identities/descriptors from catalog orchestration so language handlers no longer depend on a catalog that queries them. P009 acceptance and integration remain pending.

## Progress

Inventory, repair guidance, Rust binding analysis, delegation/environment/review ownership and declaration/use cycle correction are committed. Standard Rust module declarations remain subject to access checks but no longer manufacture use cycles. Calls, imports, type references and reexports retain cycle checks.

VCS empty-directory discovery and selective test support are committed. Architecture extraction changes select the lint test group; shared and unknown paths retain the full suite, and integration always runs full checks. Shared extraction model/context no longer invokes a language handler. The current rule change colocates Kind with its descriptor under rules, preserves rules::Kind, and changes handler imports to the independent identity owner. Rules and languages directories now have explicit inventory and dependency/public contracts.

## Verification

Commit 71c8632 passed the complete gate (68448 exited 0): 699 Python tests in 690.30 seconds and 147 Rust tests, plus all configured checks. Installed selection/configuration checks previously passed all 12 cases, including full feature merge.

Extraction commit 3a1967f passed the configured gate (50999 exited 0). Resume confirmed selective=true, completed, and full_gate_passed=false. That VAC reduced self-analysis from 36 to 35 cycles without introducing a new cycle.

Current rule ownership passed 78 catalog/configuration/language/strict-default Python cases (42178), candidate compilation and structural lint. The same self-analysis scope now reports 33 cycles and 25 missing contracts; exactly the lint/architecture/languages and lint/rules cycles disappear, with no new cycles or boundary findings. The current commit uses full fallback because shared rule/catalog and language paths are not mapped to a narrower test group.

## Blockers

No current blocker. Preserve permissions and maintained-source coverage.

## Next action

Commit rule identity ownership through the configured hook. Continue lint/scaffold ownership repairs using the remaining measured cycles. Shared root util.rs currently mixes JSON persistence, hook input access, path resolution, Git subprocesses and CLI option parsing; trace consumers before moving each responsibility. Resource bundling is also shared; preserve its actual consumers rather than moving it into one feature. Integration still requires the full gate.

Then continue worker lint/scaffold ownership and measured dependency repairs. Complete inventory and dependency checks across maintained source/tests/docs/resources and justify service/generated/third-party exclusions. The Python probe in .tmp/p009-python.yaml reports 69 unresolved local test imports and 12 missing contracts; assess explicit package imports before adding resolver modes. Existing discovery skips symlinks; canonical skills live under tooling/worker/assets/skills rather than the .agents/skills alias. Enable expanded checked policy, verify full P009 acceptance and integrate with feature-merge, retaining the branch.
