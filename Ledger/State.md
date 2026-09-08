# State

## Focus

Deliver active P009: complete checked architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Full acceptance and integration remain unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 4b7154e

Installed runtime 9aee4ae remains pinned. No merge or rebase is in progress. Examples VAC was accepted by commit gate 90573 with terminal 0. The tree was clean before the current review resource VAC.

Current uncommitted work adds five architecture contracts covering the review root, config, contracts, projects and prompts. Existing source and fixture contracts remain intact. Public entries correspond to existing library consumers and resources embedded by scaffold/package. Generated review runtime, reports and target are already excluded by review/.gitignore.

## Progress

Behavior tests now live with their feature owners across lint, review, delegation, hooks, configuration, setup, memory, gate, evidence, VCS and upgrades. Shared fixtures no longer import test modules. Old tooling/tests was evacuated and removed; exact invariant oracles and decision application links follow marked test functions. Git implementation moved unchanged into scaffold/git/mod.rs beside its tests.

Accepted examples VAC inventories seven directories, including source-free parents and hidden .gitignore files, repairs PORTABILITY.md links to current test owners and selects delivery tests for example changes. The actual example Rust root is currently configured only in .tmp/p009-combined.yaml; retain it when enabling the checked rule.

P010 remains separately planned and pending; do not implement it instead of P009 or modify voxel-rust.

## Verification

Commit d215bbd passed gate 8951: 348 selected Python tests in 549.88 seconds and all required checks. Commit 4b7154e passed gate 90573: 94 Rust tests, 29 selected Python tests in 161.08 seconds, review tests and remaining checks. These are commit selections, not full integration acceptance.

Focused example delivery run 4948 passed all six scenarios in 167.15 seconds. All local PORTABILITY.md links resolve; config-check passes.

Current combined candidate architecture probe returns [] with exit 0 after adding the full review tree to its includes. It covers worker source, relocated behavior tests, examples and review documentation/resources. Main tooling/worker/lint.yaml still does not enable architecture. This partial scope does not establish full P009 acceptance.

## Blockers

No operational blocker. Preserve behavior, exact executable oracles and full maintained-tree scope. Do not weaken thresholds, permissions or exclusions to silence findings. Root agentrig.yaml is strict; do not extract unsupported configuration packages or change the installed pin as a workaround.

## Next action

Finish the review resource VAC: inspect the five contracts and public resource boundaries, verify structural lint, stage with this State and commit through the normal gate. Do not repeat the completed examples gate.

Then cover remaining maintained root files, Ledger, Project, tooling documents and canonical skill resources. assets/skills currently contains 15 skills and .agents/skills is a single symlink to it: inspect ownership and consumers before any grouping, preserve canonical instruction bodies and installed names, and avoid arbitrary count-based folders or duplicate instructions.

Declare full checked scope and explicit justified service/generated/third-party exclusions, retain actual language roots, repair remaining measured findings, enable architecture in tooling/worker/lint.yaml, verify four-language consumers and both binaries, finish acceptance and feature-merge while retaining the branch. Do not mark P009 complete from a subset probe.
