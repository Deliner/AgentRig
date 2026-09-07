# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance is unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: f00ec04

Commit test policy is corrected in f00ec04; installed runtime 9aee4ae is available. The current P009 VAC moves atomic JSON persistence from mixed util.rs to shared artifacts and consolidates the identical check-evidence write sequence. P009 acceptance and integration remain pending.

## Progress

Inventory, repair guidance, Rust binding analysis, delegation/environment/review ownership and declaration/use cycle correction are committed. Standard Rust module declarations remain subject to access checks but no longer manufacture use cycles. Calls, imports, type references and reexports retain cycle checks.

Extraction model/context and rule identities have independent owners with preserved public paths. Commit checks use affected groups plus smoke tests, never automatic full fallback; merge remains full. The current persistence move directs jobs, delegation and check evidence to artifacts/json.rs while retaining util::save_json as a compatibility reexport. The serialization, file sync and replacement order is unchanged. Delegation permissions now name the actual JSON owner instead of util.rs. Artifact-owned tests cover failed serialization, replacement, symlink targets and missing parents.

## Verification

Commit f00ec04 passed the corrected gate (86019 exited 0). The saved parent command logs confirm SELECT [test] named only three smoke targets: six cases passed in 0.14 seconds. The complete commit command took 40.33 seconds. Full-suite fallback is removed from repository commit policy; installed full-merge behavior was previously verified.

Extraction commit 3a1967f passed the configured gate (50999 exited 0). Resume confirmed selective=true, completed, and full_gate_passed=false. That VAC reduced self-analysis from 36 to 35 cycles without introducing a new cycle.

Current persistence ownership passed four artifact Rust tests and 23 Python delegation/job/evidence cases (29094, 26.58 seconds), candidate compilation, both Rust formatters and structural lint. The same self-analysis scope reports 32 cycles and 25 missing contracts; exactly the root/delegate/run/root cycle disappears, with no new cycles or boundary findings. The affected commit gate is next; no full-suite run is required on commit.

## Blockers

The selective commit gate 54442 was cancelled (owned job run-gmIK1B, exit 143) after discovering overlapping pytest targets could omit cases; it did not commit or pass. Collecting test_commands.py alone yielded 28 cases, but adding its smoke node yielded only one. The working-tree correction uses the whole configuration test file for smoke and the same upgrades directory target across groups. Collection-only verification of all 27 unique configured targets matched separate collections exactly: 698 cases, none missing or extra (22645, exit 0). No full suite was executed. Stage this correction and CHECKS.md with the pending VAC before retrying the selective commit gate.

No current blocker. Preserve permissions and maintained-source coverage.

## Next action

Commit artifact JSON ownership through the affected gate, then continue utility and lint/scaffold ownership. Hook object/text helpers are hook-owned; path resolution and option parsing have multiple consumers; Git queries belong with the VCS owner. Hooks and scaffold currently compile in the binary, while util.rs belongs to the library: account for that boundary when preserving public consumers rather than introducing duplicated implementations or unsupported path attributes. Preserve behavior and do not create another mixed shared bucket. Integration still requires the full gate.

Then continue worker lint/scaffold ownership and measured dependency repairs. Complete inventory and dependency checks across maintained source/tests/docs/resources and justify service/generated/third-party exclusions. The Python probe in .tmp/p009-python.yaml reports 69 unresolved local test imports and 12 missing contracts; assess explicit package imports before adding resolver modes. Existing discovery skips symlinks; canonical skills live under tooling/worker/assets/skills rather than the .agents/skills alias. Enable expanded checked policy, verify full P009 acceptance and integrate with feature-merge, retaining the branch.
