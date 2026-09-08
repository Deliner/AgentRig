# State

## Focus

Deliver active P009: complete checked architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Full acceptance and integration remain unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: d800ad5

Installed runtime 9aee4ae remains pinned. No merge or rebase is in progress. Current unfinished VAC splits former test_commands.py into scaffold/commands/tests for configured execution and managed job behavior, plus scaffold/config/tests for invalid configuration diagnostics. Existing background helpers live in commands/tests/consumer.py and the old Git tests import that public fixture. Command targets follow all three scenario files; no scenario bodies or assertions changed.

## Progress

Expanded inventories, repair guidance, four-language dependency analysis, Rust binding and literal module-path handling are committed. Review, lint, delegate, upgrade, launcher and setup capability tests are colocated with their owners. Shared fixtures remain with their actual consumers' capabilities.

Latest accepted VAC d800ad5 colocates hook and reminder tests unchanged with hooks. Previous ab32505 colocated setup/package scenarios and 0d77816 colocated memory/plan/State scenarios.

The current command tests were initially put under jobs, but their scaffold project fixture created measured directory cycles because scaffold already uses jobs. These scenarios exercise configured command orchestration, so their correct owner is scaffold/commands. The lower jobs module and its original contract remain unchanged. The abandoned directory contained only two generated .pyc files; it was verified and removed.

Temporary fixture exposure across src/scaffold remains for actual external gate/feedback/Git callers. Remove unnecessary outer entries after remaining consumers move inside their owners. Hook tests still use the shared scaffold project builder across its public boundary.

P010 is separately planned and pending for configuration hook deadlock. Do not implement it instead of P009 or modify voxel-rust.

## Verification

Commit d800ad5 passed normal gate 87409 (terminal 0): 94 Rust tests, 418 selected Python tests in 463.99 seconds, review tests and all remaining checks. Earlier ab32505 passed gate 63045 with 418 Python tests. These are commit selections, not full P009 acceptance.

Current commands/config VAC passes 29 focused command/config/Git-cleanup scenarios in 5.33 seconds (42137 terminal 0), mypy on 66 sources, Ruff and formatting. Earlier focused run 74072 passed 29 scenarios before the ownership correction. All 22 moved test/helper function ASTs match HEAD and pytest still collects 701 cases in 0.56 seconds. Final structural lint and memory check 65744 pass after the location correction. The combined architecture probe confirms the new directory cycles are gone; only the previously recorded missing contracts/example roots remain.

Combined architecture probe .tmp/p009-combined.yaml has no new dependency findings. It still reports ten missing contracts under old tooling/tests and examples, plus two incomplete Rust-root findings for examples/rust/crates/engine/src/lib.rs. The probe covers only selected maintained paths; source extensions select dependency analysis, not inventory coverage. Main checked tooling/worker/lint.yaml still does not enable architecture.

## Blockers

No operational blocker. Preserve behavior, exact executable oracles and the full maintained-tree scope. Root agentrig.yaml is strict; do not extract unsupported configuration packages or change the installed pin as a workaround.

## Next action

Finish the current commands/config VAC: inspect and stage only its paths, and commit through the normal gate. Refresh this snapshot with terminal evidence. Do not begin another implementation VAC before this one is accepted.

Then colocate remaining scaffold tests by actual owner: gate, Git delivery and workflow composition. Feedback mixes evidence, resume and integration scenarios; split by behavior rather than copying helpers or moving the whole file under one misleading owner. Shared repository construction and revision observation have real gate/feedback/Git callers. Preserve exact oracles, decision markers, pytest discovery and file-relative fixture paths.

Remove evacuated old test directories after verifying only generated caches remain; remove obsolete command inputs and temporary external fixture exposure. Broaden contracts to maintained docs, resources and examples with explicit justified service/generated/third-party exclusions. Configure the example Rust crate root and other maintained language roots. Enable architecture in checked lint.yaml, verify both binaries and all four languages, complete full P009 acceptance and feature-merge, retaining the branch. A passing subset probe is not completion.
