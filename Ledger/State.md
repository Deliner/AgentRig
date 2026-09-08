# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance and integration are unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 48ee237

Installed runtime 9aee4ae remains pinned. Composition, jobs and verification evidence contracts are committed. Current VAC colocates the review MCP server and its argument schema and registers review composition, MCP and VCS boundaries. The current Rust probe has no measured cycles.

## Progress

Expanded inventories, repair guidance, four-language dependency analysis and measured Rust binding fixes are committed. Module declarations retain access checks without manufacturing use cycles; imports, calls, types and reexports retain cycle checks.

Delegate, environment, review, lint configuration/discovery, resources, arguments, paths, hook input, command execution, memory validation, settings, receipts and recovery have clearer ownership. Existing public compatibility exports remain. D005 follows command execution and settings validation; D023 follows receipt schema and generation.

Installation/mod.rs owns the unchanged serializable file State, confined file observation and atomic durable replacement formerly under recovery/model and upgrade/storage. Setup reconciliation calls installation directly. Update storage and recovery schemas use the same owner, preserving their internal exports and serialized fields. Installation uses static settings path validation, avoiding Context/recovery coupling. Package legacy rejection uses recovery::LEGACY_FILE directly. Its architecture contract and recovery permissions describe these actual dependencies.

Setup input now imports settings, package generation and manifest helpers from their actual owners instead of setup's aliases. Moving input.rs into the existing input directory colocates its data and orchestration with the handlers that use them; Rust module paths and call behavior remain unchanged. The contract registers all six files and measured dependencies.

Current recovery/storage.rs preserves payload checksums, atomic JSON persistence and recorded-file restoration; recovery/report.rs preserves plan text, Git diff invocation, mode reporting and temporary-file cleanup. Release and configuration plans use this common owner. Configuration planning now imports saved model/storage/reporting directly and its handlers share the same directory. No schema, CLI or diff behavior changed.

Commit test selection correction is accepted as 546b6d4. Python groups follow scaffold behavior ownership; Rust uses existing libtest module filters; unrelated review tests skip. Shared configuration and dispatch still select broad consumer groups. Full integration checks remain unfiltered. Explicit false command defaults are owned by the runtime schema and catchall smoke coverage is not duplicated.

Initialization now belongs to setup/init.rs and setup/wizard.rs. Package generation no longer calls setup; executors read prepared or installed resources through the package owner. CLI dispatch reaches setup's init/run entry points directly. Collision checks, creation permissions, generated-bundle validation, confirmation and cancellation bodies are preserved. New package and setup contracts register exact files, child roles and existing external consumers.

## Verification

Support contract commit 48ee237 passed gate 37688 (terminal 0): 32 Rust tests, 491 selected Python tests in 502.72 seconds and all configured checks. Review tests skipped. Current review ownership passes review-test (44496 terminal 0), including MCP tool and argument behavior, candidate release build (56058), three native delegation MCP consumers in 0.65 seconds, review Clippy and git diff --check. Probe .tmp/p009-review-composition.json reports three missing scaffold contracts and no other findings. Review commit gate is pending. The argument schema moved unchanged; no stale references to schemas/arguments.json or review/src/mcp.rs remain under tooling or Ledger.

Root composition commit 87a9f85 passed gate 89940 (terminal 0): 92 Rust tests, 251 selected Python tests in 29.29 seconds and all configured checks. Review tests skipped. This supersedes its pending gate below. Current composition/jobs/evidence contracts pass the focused Rust probe .tmp/p009-support-contracts.json with six missing contracts and no other findings; no behavior code changed. Remaining Rust contracts are review/src, review/src/mcp, review/src/vcs, scaffold, scaffold/gate and scaffold/upgrade.

Current root composition VAC preserves the standalone entry body and binary name through an explicit Cargo target at lint/standalone.rs. Both frontend consumers pass 56 native portable/architecture cases after candidate build; Clippy passes (9515 terminal 0). Probe .tmp/p009-root-owner.json reports nine missing Rust contracts and no other findings. Commit gate is pending. P010 planning commit ee20457 passed its selected gate with 23 Python configuration cases; implementation remains pending.

Memory/commands commit 98f2d73 passed its required gate (66273 terminal 0): 418 Python tests in 465.01 seconds, seven selected Rust tests and all other configured checks. Review tests were skipped as unrelated. Its architecture probe had six cycles and 15 missing contracts.

Current installation extraction passes five existing Rust receipt/recovery tests (94018). Twenty focused native setup preservation, conflicts, post-preview edits, upgrade and rollback cases passed in 39.39 seconds (13981). Two legacy-init rejection cases passed in 8.49 seconds after the final constant-owner correction (53603). Configuration, structural lint and git diff --check pass.

Installation commit 041f5a0 passed gate 66711 (terminal 0): 418 Python cases in 466.02 seconds, 92 Rust tests and all other configured checks including review. No integration has run.

Current setup input change passes 29 native package/composed-preview consumers in 18.92 seconds (7893), structural lint and git diff --check. Self-analysis .tmp/p009-setup-input.json reports three cycles, 14 missing contracts and no other findings. Direct input-to-setup coupling is removed; a longer representative through package generation remains within the existing package/setup cycle. Do not claim the entire input dependency graph is acyclic. Initial gate 93759 failed Clippy on the now-unused parent manifest import; removing that alias passed the reported staged Clippy rerun (83558). Normal commit retry is pending.

Setup input commit 055accf passed its normal retry (13122 terminal 0): 226 Python tests in 104.56 seconds, seven selected Rust tests and all other configured checks. Review tests were skipped. This supersedes the pending retry above.

Current saved-plan ownership passes Clippy (20636), 34 native release/configuration update and recovery cases in 60.63 seconds (13750), structural lint and git diff --check. Self-analysis .tmp/p009-configuration-owner.json reports two remaining package/setup cycle paths, 13 missing Rust contracts and no other findings. The upgrade/configuration cycle is removed. Current commit gate remains pending.

Saved-plan commit 5b98a57 passed gate 43397 (terminal 0): 239 Python tests in 113.10 seconds, seven Rust tests and all other configured checks. Review tests skipped. This supersedes its pending gate above.

Current initialization ownership passes Clippy (87034), 90 native setup/package cases in 49.52 seconds (74711), structural lint and git diff --check. Self-analysis .tmp/p009-package-owner.json has zero cycles, 11 missing Rust contracts and no other findings. Existing native client discovery, diagnostics and lint catalog calls are explicitly included in the new package boundary. Current commit gate is pending.

## Blockers

P010 is planned separately at the user's request: malformed configuration blocks diagnostic reads and corrective edits through hooks, and the outer error hides the YAML cause. Reproduced with read and apply_patch events in an isolated candidate fixture; voxel-rust was not modified. Ledger/Plan/010.md defines recovery without discarding edits or bypassing normal verification. Implementation remains pending.

No current operational blocker. Keep the full P009 scope and preserve behavior, permissions and maintained-source coverage.

Root agentrig.yaml is strictly decoded; setup configuration packages do not imply live root package support. A failed root-package extraction blocked all hooks until the user restored flattened checks. Do not retry that unsupported extraction or change the development pin as a workaround. Configuration is now valid at 500 nonblank lines without changing thresholds.

## Next action

Finish review composition and MCP ownership through its required commit gate, then complete scaffold, scaffold/gate and scaffold/upgrade contracts. Gate orchestration can be colocated with selection while preserving module paths and updating D004/D005/D015/D016 application links; the configured directory selector already covers that destination. Missing inventories and broader coverage still prevent full acceptance.

Complete missing Rust contracts and architecture coverage across maintained tests, docs and resources. The Python probe .tmp/p009-python.yaml still reports 69 unresolved local test imports and 12 missing contracts; inspect actual local import behavior rather than treating local code as external. Canonical skills live under tooling/worker/assets/skills; discovery skips symlinks.

Enable expanded architecture policy in the checked lint configuration, verify four-language consumers and both binaries, and complete full P009 acceptance before integration through feature-merge. Retain the feature branch. Do not hide cycles with aliases, widen permissions to silence findings, create arbitrary folders, or claim focused checks prove full acceptance.
