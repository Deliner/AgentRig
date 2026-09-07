# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance is unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 88f5c7a

Rule identity ownership is committed in 88f5c7a; installed runtime 9aee4ae is available. The user corrected the test policy: full tests belong on merge, not as a fallback for unmapped commit paths. The current maintenance VAC corrects that configuration using the existing catch-all group mechanism. Utility ownership work has not started; its preparation was read-only. P009 acceptance and integration remain pending.

## Progress

Inventory, repair guidance, Rust binding analysis, delegation/environment/review ownership and declaration/use cycle correction are committed. Standard Rust module declarations remain subject to access checks but no longer manufacture use cycles. Calls, imports, type references and reexports retain cycle checks.

Extraction model/context and rule identities have independent owners with preserved public paths. The corrected commit map always selects three smoke targets (six parametrized cases), then unions affected feature groups. It covers all lint source paths and known shared consumers. Unknown paths select only smoke tests, never the full suite. Delegation no longer selects all scaffold tests, and long project integration cases are selected only when their test file changes. Merge still runs every configured test. AGENTS and guides record this user requirement; no runtime change or new selection mechanism is needed.

## Verification

Commit 88f5c7a passed the previous full fallback gate (59468 exited 0): 699 Python tests in 703.83 seconds and 147 Rust tests. That fallback was contrary to the user's intended commit policy and is being removed. Installed selection/configuration checks previously passed all 12 cases, including full feature merge.

Extraction commit 3a1967f passed the configured gate (50999 exited 0). Resume confirmed selective=true, completed, and full_gate_passed=false. That VAC reduced self-analysis from 36 to 35 cycles without introducing a new cycle.

Rule ownership passed 78 focused cases and reduced self-analysis to 33 cycles and 25 missing contracts without new cycles or boundary findings. The corrected test policy passes installed config-check and structural lint. All six groups collect independently without import errors: 6 smoke, 251 lint, 161 delegation, 395 scaffold/upgrade, 6 project integration, and 26 review/capability cases. The policy commit itself should select only the smoke targets, proving that configuration and documentation edits no longer trigger the full Python suite.

## Blockers

No current blocker. Preserve permissions and maintained-source coverage.

## Next action

Commit the policy correction through the hook and verify SELECT [test] names only smoke targets, with selective evidence and no full-suite claim. Then resume utility ownership: JSON persistence is shared by jobs, delegation and check evidence; hook object/text helpers are hook-owned; path resolution and option parsing have multiple consumers; Git queries belong with the VCS owner. Preserve behavior and public consumers, and do not create another mixed shared bucket. Integration still requires the full gate.

Then continue worker lint/scaffold ownership and measured dependency repairs. Complete inventory and dependency checks across maintained source/tests/docs/resources and justify service/generated/third-party exclusions. The Python probe in .tmp/p009-python.yaml reports 69 unresolved local test imports and 12 missing contracts; assess explicit package imports before adding resolver modes. Existing discovery skips symlinks; canonical skills live under tooling/worker/assets/skills rather than the .agents/skills alias. Enable expanded checked policy, verify full P009 acceptance and integrate with feature-merge, retaining the branch.
