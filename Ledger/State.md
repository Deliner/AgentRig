# State

## Focus

Deliver active P009: complete checked architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Full acceptance and integration remain unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 1814a08

Installed runtime 9aee4ae remains pinned. No merge or rebase is in progress. Repository map VAC was accepted by commit gate 41855 with terminal 0. The tree was clean before the current canonical resource VAC.

Current partly staged VAC groups five memory guidance skills under assets/skills/memory and adds eighteen resource contracts. Original bodies are unchanged. Flat symlinks preserve existing hook and client paths, including .agents/skills, while Rust includes the canonical files directly. Installed names remain flat. The root contract documents these compatibility aliases and their existing discovery semantics. Other skill ownership remains unchanged.

## Progress

Behavior tests now live with their feature owners across lint, review, delegation, hooks, configuration, setup, memory, gate, evidence, VCS and upgrades. Shared fixtures no longer import test modules. Old tooling/tests was evacuated and removed; exact invariant oracles and decision application links follow marked test functions. Git implementation moved unchanged into scaffold/git/mod.rs beside its tests.

Accepted examples VAC inventories seven directories, including source-free parents and hidden .gitignore files, repairs PORTABILITY.md links to current test owners and selects delivery tests for example changes. The actual example Rust root is currently configured only in .tmp/p009-combined.yaml; retain it when enabling the checked rule.

P010 remains separately planned and pending; do not implement it instead of P009 or modify voxel-rust.

## Verification

Commit d215bbd passed gate 8951: 348 selected Python tests in 549.88 seconds and all required checks. Commit 4b7154e passed gate 90573: 94 Rust tests, 29 selected Python tests in 161.08 seconds, review tests and remaining checks. These are commit selections, not full integration acceptance.

Focused example delivery run 4948 passed all six scenarios in 167.15 seconds. All local PORTABILITY.md links resolve; config-check passes.

Commit 335c167 passed gate 86663, including selected Rust/Python checks, review tests and all remaining checks. Commit 1814a08 passed gate 41855 with 23 Python smoke cases in 0.13 seconds and remaining required checks (terminal 0).

The combined candidate probe selects the entire repository with . and ** and now returns [] (exit 0), including canonical resources and every crossed resource boundary. Before staging renames Git still listed removed files behind compatibility symlinks; staging the actual moves resolved those stale inventory paths without contract exceptions. Structural lint passes. Focused manifest/resource run 12058 passed all 13 cases in 6.87 seconds. A fresh candidate installation independently verified all 13 base skill names and byte-for-byte contents against HEAD; all five old client paths resolve to unchanged canonical bodies. Main tooling/worker/lint.yaml still does not enable architecture, and full P009 acceptance remains unfinished.

## Blockers

No operational blocker. Preserve behavior, exact executable oracles and full maintained-tree scope. Do not weaken thresholds, permissions or exclusions to silence findings. Root agentrig.yaml is strict; do not extract unsupported configuration packages or change the installed pin as a workaround.

## Next action

Finish the canonical resource VAC: inspect staged moves and public resource contracts, stage remaining comments and this State, verify memory and commit through the normal gate. Do not repeat completed focused installation checks.

Then promote the full-tree architecture rule from the temporary probe into checked lint.yaml. Review explicit generated/service exclusions, document checked scope and skill compatibility paths, and repair remaining stale documentation links to relocated tests. Preserve current thresholds and actual source roots.

Declare full checked scope and explicit justified service/generated/third-party exclusions, retain actual language roots, repair remaining measured findings, enable architecture in tooling/worker/lint.yaml, verify four-language consumers and both binaries, finish acceptance and feature-merge while retaining the branch. Do not mark P009 complete from a subset probe.
