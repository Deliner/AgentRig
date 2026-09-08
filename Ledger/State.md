# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance is unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: b77e5fd

Installed runtime 9aee4ae is available. Contextual Git query ownership is committed in b77e5fd. The current P009 VAC consolidates commit/reference guards into the existing scaffold/git.rs delivery owner and removes scaffold's dependency on hooks. P009 acceptance and integration remain pending.

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

Shared path ownership was committed in eaee21b through the selective gate (89697 exited 0): 418 Python cases in 506.78 seconds plus configured Rust and other checks. Its self-analysis left 25 cycles and 18 missing contracts.

Current Git ownership preserves inherited GIT_DIR/GIT_WORK_TREE, trimmed UTF-8 output and raw Git stderr failures; the existing isolated VCS runner retains its separate environment semantics. Hooks and jobs call the VCS owner directly. Two colocated Rust tests pass after extracting repository setup into a helper; structural lint reports no errors. Two native command/rebase tests pass (75400, 14.67 seconds). Candidate self-analysis in .tmp/p009-git-owner.json removes the root/delegate/run/jobs/root cycle without new cycles, leaving 24 cycles and 18 missing contracts. Commit verification is pending.

Git query ownership was committed in b77e5fd after correcting inline test placement. The retry (32967 exited 0) passed 692 Python cases in 546.88 seconds, Rust tests, Clippy and all other configured checks.

Current delivery guard consolidation keeps the original function bodies, updates their two scaffold callers and migrates D019's current application link. Three Git/Mercurial consumer cases pass (47203, 55.97 seconds). The existing divergent-rebase test now also attempts deletion of the integrated feature and verifies that the guard rejects it and preserves the reference; this updated case passes separately (34988, 2.46 seconds). Structural lint reports no errors. Candidate self-analysis in .tmp/p009-delivery-guards.json removes the hooks/scaffold/hooks cycle without new cycles or other findings, leaving 23 cycles and 18 missing contracts. Commit verification remains pending.

The first delivery guard commit attempt (88068 exited 1) passed 418 Python cases in 552.59 seconds, then failed the unchanged review execution case external_review_and_repair_preserve_revision_scope_and_workspace with a timeout before role launch. The complete staged review-test retry (93691 exited 0) passed without code changes. Commit retry remains pending; this is not a completed VAC.

## Blockers

No current blocker. Preserve permissions and maintained-source coverage.

## Next action

Commit delivery guard consolidation through the affected gate, then continue remaining contracts and utility/scaffold ownership. The user reiterated that full checks belong only on merge; current affected groups are coarse and shared VCS changes select most Python groups. Refine selection against actual consumers without replacing the mandatory gate with an ad hoc subset. Hook object/text helpers are hook-owned. Hooks and scaffold currently compile in the binary, while util.rs belongs to the library: account for that boundary when preserving public consumers rather than introducing duplicated implementations or unsupported path attributes. Preserve behavior and do not create another mixed shared bucket. Integration still requires the full gate.

Then continue worker lint/scaffold ownership and measured dependency repairs. Complete inventory and dependency checks across maintained source/tests/docs/resources and justify service/generated/third-party exclusions. The Python probe in .tmp/p009-python.yaml reports 69 unresolved local test imports and 12 missing contracts; assess explicit package imports before adding resolver modes. Existing discovery skips symlinks; canonical skills live under tooling/worker/assets/skills rather than the .agents/skills alias. Enable expanded checked policy, verify full P009 acceptance and integrate with feature-merge, retaining the branch.
