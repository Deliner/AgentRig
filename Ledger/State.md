# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance and integration are unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 98f2d73

Installed runtime 9aee4ae remains pinned. The memory/commands ownership VAC is committed. Current uncommitted VAC removes setup and package generation dependencies on upgrade execution through a shared installation owner and the existing recovery constant.

## Progress

Expanded inventories, repair guidance, four-language dependency analysis and measured Rust binding fixes are committed. Module declarations retain access checks without manufacturing use cycles; imports, calls, types and reexports retain cycle checks.

Delegate, environment, review, lint configuration/discovery, resources, arguments, paths, hook input, command execution, memory validation, settings, receipts and recovery have clearer ownership. Existing public compatibility exports remain. D005 follows command execution and settings validation; D023 follows receipt schema and generation.

Current installation/mod.rs owns the unchanged serializable file State, confined file observation and atomic durable replacement formerly under recovery/model and upgrade/storage. Setup reconciliation calls installation directly. Update storage and recovery schemas use the same owner, preserving their internal exports and serialized fields. Installation uses static settings path validation, avoiding Context/recovery coupling. Package legacy rejection uses recovery::LEGACY_FILE directly. Its architecture contract and recovery permissions describe these actual dependencies.

Commit test selection correction is accepted as 546b6d4. Python groups follow scaffold behavior ownership; Rust uses existing libtest module filters; unrelated review tests skip. Shared configuration and dispatch still select broad consumer groups. Full integration checks remain unfiltered. Explicit false command defaults are owned by the runtime schema and catchall smoke coverage is not duplicated.

## Verification

Memory/commands commit 98f2d73 passed its required gate (66273 terminal 0): 418 Python tests in 465.01 seconds, seven selected Rust tests and all other configured checks. Review tests were skipped as unrelated. Its architecture probe had six cycles and 15 missing contracts.

Current installation extraction passes five existing Rust receipt/recovery tests (94018). Twenty focused native setup preservation, conflicts, post-preview edits, upgrade and rollback cases passed in 39.39 seconds (13981). Two legacy-init rejection cases passed in 8.49 seconds after the final constant-owner correction (53603). Configuration, structural lint and git diff --check pass.

Current self-analysis .tmp/p009-installation.json reports three cycles, 15 missing contracts and no other findings. Setup and package generation no longer depend on upgrade. Remaining cycles are package/setup, setup/input and upgrade/configuration. Current commit gate has not yet passed; no integration has run.

## Blockers

No current operational blocker. Keep the full P009 scope and preserve behavior, permissions and maintained-source coverage.

Root agentrig.yaml is strictly decoded; setup configuration packages do not imply live root package support. A failed root-package extraction blocked all hooks until the user restored flattened checks. Do not retry that unsupported extraction or change the development pin as a workaround. Configuration is now valid at 500 nonblank lines without changing thresholds.

## Next action

Inspect and commit the installation ownership VAC through the required staged hook. Then repair the three remaining measured cycles with complete ownership moves: package currently mixes wizard orchestration and generated-file access with bundle generation; setup input imports parent setup facades; upgrade configuration imports its parent workflow.

Complete missing Rust contracts and architecture coverage across maintained tests, docs and resources. The Python probe .tmp/p009-python.yaml still reports 69 unresolved local test imports and 12 missing contracts; inspect actual local import behavior rather than treating local code as external. Canonical skills live under tooling/worker/assets/skills; discovery skips symlinks.

Enable expanded architecture policy in the checked lint configuration, verify four-language consumers and both binaries, and complete full P009 acceptance before integration through feature-merge. Retain the feature branch. Do not hide cycles with aliases, widen permissions to silence findings, create arbitrary folders, or claim focused checks prove full acceptance.
