# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance and integration are unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 5b98a57

Installed runtime 9aee4ae remains pinned. Shared saved-plan ownership is committed. Current VAC moves initialization and its wizard into setup, keeps prepared/installed resource reading with package generation, and records both architecture boundaries. The current Rust probe has no measured cycles.

## Progress

Expanded inventories, repair guidance, four-language dependency analysis and measured Rust binding fixes are committed. Module declarations retain access checks without manufacturing use cycles; imports, calls, types and reexports retain cycle checks.

Delegate, environment, review, lint configuration/discovery, resources, arguments, paths, hook input, command execution, memory validation, settings, receipts and recovery have clearer ownership. Existing public compatibility exports remain. D005 follows command execution and settings validation; D023 follows receipt schema and generation.

Installation/mod.rs owns the unchanged serializable file State, confined file observation and atomic durable replacement formerly under recovery/model and upgrade/storage. Setup reconciliation calls installation directly. Update storage and recovery schemas use the same owner, preserving their internal exports and serialized fields. Installation uses static settings path validation, avoiding Context/recovery coupling. Package legacy rejection uses recovery::LEGACY_FILE directly. Its architecture contract and recovery permissions describe these actual dependencies.

Setup input now imports settings, package generation and manifest helpers from their actual owners instead of setup's aliases. Moving input.rs into the existing input directory colocates its data and orchestration with the handlers that use them; Rust module paths and call behavior remain unchanged. The contract registers all six files and measured dependencies.

Current recovery/storage.rs preserves payload checksums, atomic JSON persistence and recorded-file restoration; recovery/report.rs preserves plan text, Git diff invocation, mode reporting and temporary-file cleanup. Release and configuration plans use this common owner. Configuration planning now imports saved model/storage/reporting directly and its handlers share the same directory. No schema, CLI or diff behavior changed.

Commit test selection correction is accepted as 546b6d4. Python groups follow scaffold behavior ownership; Rust uses existing libtest module filters; unrelated review tests skip. Shared configuration and dispatch still select broad consumer groups. Full integration checks remain unfiltered. Explicit false command defaults are owned by the runtime schema and catchall smoke coverage is not duplicated.

Initialization now belongs to setup/init.rs and setup/wizard.rs. Package generation no longer calls setup; executors read prepared or installed resources through the package owner. CLI dispatch reaches setup's init/run entry points directly. Collision checks, creation permissions, generated-bundle validation, confirmation and cancellation bodies are preserved. New package and setup contracts register exact files, child roles and existing external consumers.

## Verification

Memory/commands commit 98f2d73 passed its required gate (66273 terminal 0): 418 Python tests in 465.01 seconds, seven selected Rust tests and all other configured checks. Review tests were skipped as unrelated. Its architecture probe had six cycles and 15 missing contracts.

Current installation extraction passes five existing Rust receipt/recovery tests (94018). Twenty focused native setup preservation, conflicts, post-preview edits, upgrade and rollback cases passed in 39.39 seconds (13981). Two legacy-init rejection cases passed in 8.49 seconds after the final constant-owner correction (53603). Configuration, structural lint and git diff --check pass.

Installation commit 041f5a0 passed gate 66711 (terminal 0): 418 Python cases in 466.02 seconds, 92 Rust tests and all other configured checks including review. No integration has run.

Current setup input change passes 29 native package/composed-preview consumers in 18.92 seconds (7893), structural lint and git diff --check. Self-analysis .tmp/p009-setup-input.json reports three cycles, 14 missing contracts and no other findings. Direct input-to-setup coupling is removed; a longer representative through package generation remains within the existing package/setup cycle. Do not claim the entire input dependency graph is acyclic. Initial gate 93759 failed Clippy on the now-unused parent manifest import; removing that alias passed the reported staged Clippy rerun (83558). Normal commit retry is pending.

Setup input commit 055accf passed its normal retry (13122 terminal 0): 226 Python tests in 104.56 seconds, seven selected Rust tests and all other configured checks. Review tests were skipped. This supersedes the pending retry above.

Current saved-plan ownership passes Clippy (20636), 34 native release/configuration update and recovery cases in 60.63 seconds (13750), structural lint and git diff --check. Self-analysis .tmp/p009-configuration-owner.json reports two remaining package/setup cycle paths, 13 missing Rust contracts and no other findings. The upgrade/configuration cycle is removed. Current commit gate remains pending.

Saved-plan commit 5b98a57 passed gate 43397 (terminal 0): 239 Python tests in 113.10 seconds, seven Rust tests and all other configured checks. Review tests skipped. This supersedes its pending gate above.

Current initialization ownership passes Clippy (87034), 90 native setup/package cases in 49.52 seconds (74711), structural lint and git diff --check. Self-analysis .tmp/p009-package-owner.json has zero cycles, 11 missing Rust contracts and no other findings. Existing native client discovery, diagnostics and lint catalog calls are explicitly included in the new package boundary. Current commit gate is pending.

## Blockers

No current operational blocker. Keep the full P009 scope and preserve behavior, permissions and maintained-source coverage.

Root agentrig.yaml is strictly decoded; setup configuration packages do not imply live root package support. A failed root-package extraction blocked all hooks until the user restored flattened checks. Do not retry that unsupported extraction or change the development pin as a workaround. Configuration is now valid at 500 nonblank lines without changing thresholds.

## Next action

Commit initialization ownership through the required staged hook. Then complete the remaining Rust contracts without introducing cycles or arbitrary directory splits. The expanded Rust probe currently measures no cycles, but missing inventories and broader coverage still prevent full acceptance.

Complete missing Rust contracts and architecture coverage across maintained tests, docs and resources. The Python probe .tmp/p009-python.yaml still reports 69 unresolved local test imports and 12 missing contracts; inspect actual local import behavior rather than treating local code as external. Canonical skills live under tooling/worker/assets/skills; discovery skips symlinks.

Enable expanded architecture policy in the checked lint configuration, verify four-language consumers and both binaries, and complete full P009 acceptance before integration through feature-merge. Retain the feature branch. Do not hide cycles with aliases, widen permissions to silence findings, create arbitrary folders, or claim focused checks prove full acceptance.
