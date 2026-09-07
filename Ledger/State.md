# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance is unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 71c8632

Runtime promotion and affected-test policy are committed in 71c8632; installed runtime 9aee4ae is available. The current P009 VAC separates shared extraction records/context from language dispatch and gives Rust normalization to the Rust handler. P009 acceptance and integration remain pending.

## Progress

Inventory, repair guidance, Rust binding analysis, delegation/environment/review ownership and declaration/use cycle correction are committed. Standard Rust module declarations remain subject to access checks but no longer manufacture use cycles. Calls, imports, type references and reexports retain cycle checks.

VCS empty-directory discovery and selective test support are committed. Architecture/lint changes select 222 Python tests; shared and unknown paths retain the full suite, and integration always runs full checks. The current extraction change preserves public source type reexports and all normalization call sites. Shared model/context no longer invokes a language handler. Source, model and Rust directories now have explicit inventory and public/dependency contracts.

## Verification

Commit 71c8632 passed the complete gate (68448 exited 0): 699 Python tests in 690.30 seconds and 147 Rust tests, plus all configured checks. Installed selection/configuration checks previously passed all 12 cases, including full feature merge.

Current extraction ownership passed 51 Rust architecture tests (62473), 31 Python macro/attribute/binding cases, candidate compilation and structural lint. Comparing identical self-analysis scope against an isolated HEAD export shows 36 cycles before and 35 after: exactly source -> source/rust -> source is removed, with no new cycles. Missing contracts decrease from 29 to 27. Older 34-cycle evidence preceded the gate-selection implementation and is not the current baseline. The mandatory selective commit gate is next.

## Blockers

No current blocker. Preserve permissions and maintained-source coverage.

## Next action

Commit extraction ownership through the configured hook and confirm its selected 222-test group. Do not claim full-suite evidence from a selective run. Continue lint/scaffold ownership repairs using the remaining measured cycles; integration still requires the full gate.

Then continue worker lint/scaffold ownership and measured dependency repairs. Complete inventory and dependency checks across maintained source/tests/docs/resources and justify service/generated/third-party exclusions. The Python probe in .tmp/p009-python.yaml reports 69 unresolved local test imports and 12 missing contracts; assess explicit package imports before adding resolver modes. Existing discovery skips symlinks; canonical skills live under tooling/worker/assets/skills rather than the .agents/skills alias. Enable expanded checked policy, verify full P009 acceptance and integrate with feature-merge, retaining the branch.
