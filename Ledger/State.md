# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance is unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: dece9c0

Installed runtime 9aee4ae is available. Shared argument ownership is committed in dece9c0. The current P009 VAC moves shared path resolution from mixed util.rs to paths/mod.rs with colocated behavior tests and an exact contract. P009 acceptance and integration remain pending.

## Progress

Inventory, repair guidance, Rust binding analysis, delegation/environment/review ownership and declaration/use cycle correction are committed. Standard Rust module declarations remain subject to access checks but no longer manufacture use cycles. Calls, imports, type references and reexports retain cycle checks.

Extraction model/context, rule identities and atomic JSON persistence have independent owners with preserved public paths. Commit checks use affected groups plus configuration smoke tests, never automatic full fallback; merge remains full.

Lint configuration, inventory and selection now own their implementations in separate modules. The catalogue composes static rule definitions with language capabilities as a sibling owner; it must not live inside the static rules boundary, which language handlers depend on. Architecture settings belong with rule definitions, and rules.rs preserves explicit public exports. Existing Rust module paths, CLI and YAML behavior remain unchanged. Current decision application links follow relocated marked owners. Six new contracts describe these owners and the lint/architecture parents with exact inventories and public dependencies.

## Verification

Commit d4f1a21 passed its selective gate (27399 exited 0): 493 Python cases in 526.72 seconds plus configured Rust and other checks. Its wide affected set covered delegation, jobs, setup and evidence; it did not run the full suite. Pytest selection uses nonoverlapping whole targets after a measured file/node overlap was corrected. Full-merge behavior remains unchanged.

Commit e042b69 passed the selective gate (4296 exited 0): 251 Python cases in 31.90 seconds plus configured Rust and other checks. It removed three lint cycles without adding new ones.

Commit 66e5dc7 passed its selective gate (58310 exited 0). Resolver implementations and their tests share owners; all resolver/runner directories now have contracts. Its self-analysis reported 29 cycles and 18 missing contracts.

Commit 8d4990c passed its selective gate (16954 exited 0): 491 Python cases in 588.90 seconds and configured Rust checks. It removed two root/delegation cycles, leaving 27 cycles and 18 missing contracts.

Commit dece9c0 passed its selective gate (80052 exited 0): 617 Python cases in 514.16 seconds and configured Rust checks. It removed the lint/root cycle, leaving 26 cycles and 18 missing contracts.

Current path ownership preserves util::resolve as an explicit compatibility export; hooks, configuration, memory, setup and migration use the actual owner. The resolver still permits missing tails and leaves project-boundary enforcement to callers. Three path tests pass (24586), covering normalization, relative/absolute links, dangling links and cycles. Five hook/interactive setup cases pass in 5.96 seconds (97934). Candidate self-analysis (92241, .tmp/p009-paths-owner.json) removes one root/hooks/scaffold/memory/upgrade/root cycle without new cycles or other findings, leaving 25 cycles and 18 missing contracts. Scaffold's affected map covers paths/**. Final selective commit verification remains pending.

## Blockers

No current blocker. Preserve permissions and maintained-source coverage.

## Next action

Commit shared path ownership through the affected gate, then continue remaining contracts and utility/scaffold ownership. Hook object/text helpers are hook-owned; Git queries belong with the VCS owner. Hooks and scaffold currently compile in the binary, while util.rs belongs to the library: account for that boundary when preserving public consumers rather than introducing duplicated implementations or unsupported path attributes. Preserve behavior and do not create another mixed shared bucket. Integration still requires the full gate.

Then continue worker lint/scaffold ownership and measured dependency repairs. Complete inventory and dependency checks across maintained source/tests/docs/resources and justify service/generated/third-party exclusions. The Python probe in .tmp/p009-python.yaml reports 69 unresolved local test imports and 12 missing contracts; assess explicit package imports before adding resolver modes. Existing discovery skips symlinks; canonical skills live under tooling/worker/assets/skills rather than the .agents/skills alias. Enable expanded checked policy, verify full P009 acceptance and integrate with feature-merge, retaining the branch.
