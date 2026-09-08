# State

## Focus

Deliver active P009: complete checked architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Full acceptance and integration remain unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 086b9db

Installed runtime 9aee4ae remains pinned. No merge or rebase is in progress. Current unfinished VAC completes the remaining old native test moves: gate selection and revision scenarios belong to gate, evidence freshness to evidence, resume and revision-history checks to memory, feature/transaction scenarios to git, private setup preview to setup, and independent consumer delivery to scaffold/testing.

Git implementation moved unchanged from scaffold/git.rs to scaffold/git/mod.rs to colocate its behavior tests. Exact configured oracles and decision application links follow the original marked functions. The VAC is staged. Prior commit gate 84404 was interrupted: its handle is missing, no commit/pytest process remains, and checks.json records unfinished staged selective verification after build, lint, memory, rustfmt and Clippy passed. HEAD still has no commit for this VAC; retry its normal commit gate.

## Progress

Prior commits colocated lint, review, delegate, upgrade, distribution, setup/package, memory, hooks and configured command tests. Latest accepted 086b9db keeps configured job scenarios with scaffold/commands; placing these CLI consumers under lower-level jobs would create a real jobs-to-scaffold dependency cycle.

Current shared fixtures no longer import test modules. Basic empty memory construction moved into the existing scaffold/testing/consumer.py project builder. Committed decision baselines remain in memory/tests/consumer.py. New testing/repository.py combines existing repository setup, GATE configuration, revision/commit observations and native Mercurial initialization helpers. The shared factory depends only on the basic project fixture.

Old tooling/tests contained only generated Python caches after the moves; contents were verified and the evacuated directory removed. Mypy and Vulture no longer name that obsolete input. Affected targets follow every split scenario and collapse into directory targets only where every current test file was already selected. This preserves current test coverage. Temporary outer public entries for memory, setup and command fixtures were removed after their external consumers moved inside scaffold. Shared project construction remains public for real hook/review consumers.

P010 remains separately planned and pending. Do not implement it instead of P009 or modify voxel-rust.

## Verification

Commit 086b9db passed normal gate 97338 (terminal 0): 94 Rust tests, 418 selected Python tests in 460.15 seconds, review tests and all remaining checks. Earlier d800ad5 passed gate 87409 with 418 Python tests. These are commit selections, not full P009 acceptance.

Current VAC passes mypy on 73 sources, Ruff, formatting, structural lint and memory check 23686. Candidate compiled after the Rust file move. Pytest still collects 701 cases in 0.67 seconds. All 62 moved scenario-function ASTs match HEAD after hoisting imports and adjusting the delivery example-root depth from parents[3] to parents[4]. The native git source body remains unchanged.

Focused behavioral run 80259 passed all 169 gate, git, evidence, memory, scaffold/testing delivery and setup/test_vcs scenarios in 496.00 seconds (terminal 0). Updated State passed memory check 24382. The affected Git selector follows git/** so its colocated code and tests select their feature checks.

Combined probe .tmp/p009-combined.yaml now reports only seven missing contracts under examples and two incomplete-root findings for examples/rust/crates/engine/src/lib.rs. No remaining new dependency, private-access or cycle findings exist in that selected source/test scope. An initial private-access finding for scaffold -> git was corrected by declaring the existing mod.rs entry public. This partial include scope is not full maintained-tree acceptance; source extensions select dependency analysis, not inventory coverage. Main tooling/worker/lint.yaml still does not enable architecture.

## Blockers

No operational blocker. Preserve behavior, exact executable oracles and full maintained-tree scope. Root agentrig.yaml is strict; do not extract unsupported configuration packages or change the installed pin as a workaround.

## Next action

Finish the current remaining-test ownership VAC: inspect the full diff and stage and commit through the normal gate. Review the exact target mapping and marked oracle/application links; do not weaken selectors or boundaries. Refresh this snapshot with terminal evidence. Finish this VAC before beginning another implementation change.

Then cover maintained examples, documentation and resources with exact contracts and explicit justified service/generated/third-party exclusions. Configure the example Rust crate root and all maintained language roots. Review full-tree ownership and any remaining actual analysis limitations, enable architecture in checked lint.yaml, verify both binaries and all four languages, complete full P009 acceptance and feature-merge while retaining the branch. A passing subset probe is not completion.
