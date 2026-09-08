# State

## Focus

Deliver active P009: complete checked architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Full acceptance and integration remain unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 4e42a7f

Installed runtime 9aee4ae remains pinned. No merge or rebase is in progress. Canonical resource VAC was accepted by commit gate 37726 with terminal 0. The tree was clean before the current checked architecture enablement VAC.

Current VAC enables architecture at error severity in tooling/worker/lint.yaml for . and **, with explicit generated/service exclusions and actual Rust roots. Both binaries select all supported dependency extensions independently of inventory coverage. The repository lint check and just lint call the candidate through the existing build adapter and command catalog. Installed runtime orchestration and consumer defaults remain unchanged. Documentation describes scope, exclusions, compatibility aliases and candidate lint; stale tooling/tests/native references are repaired.

## Progress

Behavior tests now live with their feature owners across lint, review, delegation, hooks, configuration, setup, memory, gate, evidence, VCS and upgrades. Shared fixtures no longer import test modules. Old tooling/tests was evacuated and removed; exact invariant oracles and decision application links follow marked test functions. Git implementation moved unchanged into scaffold/git/mod.rs beside its tests.

Accepted examples VAC inventories seven directories, including source-free parents and hidden .gitignore files, repairs PORTABILITY.md links to current test owners and selects delivery tests for example changes. The actual example Rust root is currently configured only in .tmp/p009-combined.yaml; retain it when enabling the checked rule.

P010 remains separately planned and pending; do not implement it instead of P009 or modify voxel-rust.

## Verification

Commit d215bbd passed gate 8951: 348 selected Python tests in 549.88 seconds and all required checks. Commit 4b7154e passed gate 90573: 94 Rust tests, 29 selected Python tests in 161.08 seconds, review tests and remaining checks. These are commit selections, not full integration acceptance.

Focused example delivery run 4948 passed all six scenarios in 167.15 seconds. All local PORTABILITY.md links resolve; config-check passes.

Commit 335c167 passed gate 86663, including selected Rust/Python checks, review tests and all remaining checks. Commit 1814a08 passed gate 41855 with 23 Python smoke cases in 0.13 seconds and remaining required checks (terminal 0).

Commit 4e42a7f passed gate 37726: seven selected Rust tests, 226 Python cases in 106.84 seconds and all required checks. Five memory guidance bodies moved unchanged; original client paths and all thirteen installed base skill names/bodies were verified. Focused resource tests passed thirteen cases in 6.87 seconds.

Current checked configuration passes just check --only lint, config-check and lint-config-check. Architecture test run 58636 passed all 149 cases in 12.24 seconds, including inventory/discovery, measured boundaries and real Rust/Python/JavaScript/TypeScript consumers with both binaries. Direct repository runs of agentrig and agentrig-lint agree: no architecture findings or error-level findings, and the same 33 nonblocking structural size warnings. This verifies the checked full-tree scope, not merely the old temporary probe. Commit and integration gates for this VAC are not yet complete.

The installed runtime lint was independently tested against the full rule and fails on Rust path attributes supported by the candidate. Therefore repository lint now uses a candidate command check, preserving the installed runtime for hooks and gate orchestration under D027. No parser weakening, source exclusions or pin promotion were used to make the analysis pass.

## Blockers

No operational blocker. Preserve behavior, exact executable oracles and full maintained-tree scope. Do not weaken thresholds, permissions or exclusions to silence findings. Root agentrig.yaml is strict; do not extract unsupported configuration packages or change the installed pin as a workaround.

## Next action

Finish the checked architecture enablement VAC: inspect configuration, thin lint adapter and documentation, stage with this State and commit through the normal exported-index gate. This must verify candidate lint against the staged tree, not just the live checkout.

Then audit every P009 acceptance requirement against current code, contracts, tests and Git. Update Plan delivery evidence only when the full feature acceptance is established, commit it and run feature-merge for full integration verification. Retain the feature branch and refresh State from terminal integration evidence.

Declare full checked scope and explicit justified service/generated/third-party exclusions, retain actual language roots, repair remaining measured findings, enable architecture in tooling/worker/lint.yaml, verify four-language consumers and both binaries, finish acceptance and feature-merge while retaining the branch. Do not mark P009 complete from a subset probe.
