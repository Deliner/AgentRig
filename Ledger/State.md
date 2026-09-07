# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance is unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 66e5dc7

Installed runtime 9aee4ae is available. Resolver colocation and contracts are committed in 66e5dc7. The current P009 VAC gives frozen resource bundle preparation an independent owner with colocated behavior tests. P009 acceptance and integration remain pending.

## Progress

Inventory, repair guidance, Rust binding analysis, delegation/environment/review ownership and declaration/use cycle correction are committed. Standard Rust module declarations remain subject to access checks but no longer manufacture use cycles. Calls, imports, type references and reexports retain cycle checks.

Extraction model/context, rule identities and atomic JSON persistence have independent owners with preserved public paths. Commit checks use affected groups plus configuration smoke tests, never automatic full fallback; merge remains full.

Lint configuration, inventory and selection now own their implementations in separate modules. The catalogue composes static rule definitions with language capabilities as a sibling owner; it must not live inside the static rules boundary, which language handlers depend on. Architecture settings belong with rule definitions, and rules.rs preserves explicit public exports. Existing Rust module paths, CLI and YAML behavior remain unchanged. Current decision application links follow relocated marked owners. Six new contracts describe these owners and the lint/architecture parents with exact inventories and public dependencies.

## Verification

Commit d4f1a21 passed its selective gate (27399 exited 0): 493 Python cases in 526.72 seconds plus configured Rust and other checks. Its wide affected set covered delegation, jobs, setup and evidence; it did not run the full suite. Pytest selection uses nonoverlapping whole targets after a measured file/node overlap was corrected. Full-merge behavior remains unchanged.

Commit e042b69 passed the selective gate (4296 exited 0): 251 Python cases in 31.90 seconds plus configured Rust and other checks. It removed three lint cycles without adding new ones.

Commit 66e5dc7 passed its selective gate (58310 exited 0). Resolver implementations and their tests share owners; all resolver/runner directories now have contracts. Its self-analysis reported 29 cycles and 18 missing contracts.

The current shared resource owner preserves resources::Bundle/File/Input/digest paths. Byte hashing reuses the identical artifact digest; bundle identity still includes executable mode. Three colocated tests pass (58248): input content/mode mutation, destination collisions, directory identity and symlink rejection. Candidate self-analysis (32490, .tmp/p009-resource-owner.json) removes exactly two root/delegation cycles, leaving 27 cycles and 18 missing contracts with no new cycles or other findings. Delegation contracts name resources/mod.rs; both affected groups also include resources/** so future commits keep testing consumers. Final selective commit verification remains pending.

## Blockers

No current blocker. Preserve permissions and maintained-source coverage.

## Next action

Commit shared resource ownership through the affected gate, then continue remaining contracts and utility/scaffold ownership. Hook object/text helpers are hook-owned; path resolution and option parsing have multiple consumers; Git queries belong with the VCS owner. Hooks and scaffold currently compile in the binary, while util.rs belongs to the library: account for that boundary when preserving public consumers rather than introducing duplicated implementations or unsupported path attributes. Preserve behavior and do not create another mixed shared bucket. Integration still requires the full gate.

Then continue worker lint/scaffold ownership and measured dependency repairs. Complete inventory and dependency checks across maintained source/tests/docs/resources and justify service/generated/third-party exclusions. The Python probe in .tmp/p009-python.yaml reports 69 unresolved local test imports and 12 missing contracts; assess explicit package imports before adding resolver modes. Existing discovery skips symlinks; canonical skills live under tooling/worker/assets/skills rather than the .agents/skills alias. Enable expanded checked policy, verify full P009 acceptance and integrate with feature-merge, retaining the branch.
