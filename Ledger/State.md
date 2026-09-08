# State

## Focus

Deliver active P009: complete checked architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Full acceptance and integration remain unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 0d77816

Installed runtime 9aee4ae remains pinned. No merge or rebase is in progress. Current unfinished VAC colocates remaining setup/package/strict tests with setup and package. Existing declaration/delegated_project helpers live in setup/tests/consumer.py for setup and gate. Setup scenarios are grouped by client settings, layout, resources, declarative setup and wizard behavior; package tests cover doctor, manifest and installed lint defaults. Exact I017/I020 oracle paths and D021/D023 application links follow unchanged bodies and markers.

## Progress

Expanded inventories, repair guidance, four-language dependency analysis, Rust binding and literal module-path handling are committed. Review, lint, delegate, upgrade, launcher and setup capability tests are colocated with their owners. Shared fixtures remain with their actual consumers' capabilities.

Latest accepted VAC 0d77816 places memory, plan and State scenarios beside memory. Gate/feedback share memory/tests/consumer.py. Previous 2cc8c30 separated review CLI, setup capabilities and common scaffold consumer construction.

Current setup fixture exposure is exact: setup/tests/consumer.py is public across package, scaffold and src only for its actual remaining external gate caller. Remove unnecessary outer exposure when callers move inside their owners. Existing memory/scaffold fixture boundaries need the same cleanup after remaining test moves.

P010 is separately planned and pending for configuration hook deadlock. Do not implement it instead of P009 or modify voxel-rust.

## Verification

Commit 0d77816 passed normal gate 82431 (terminal 0): 94 Rust tests, 418 selected Python tests in 457.32 seconds, review tests and all remaining checks. The earlier review/setup commit 2cc8c30 passed gate 24312 with 499 selected Python tests. Neither proves full P009 acceptance.

Current setup/package VAC passes all 171 setup/package/gate scenarios in 113.83 seconds (2504 terminal 0), mypy on 63 sources, Ruff, formatting, structural lint and memory check 86454. All 50 moved test/helper ASTs match HEAD; pytest still collects 701 cases in 0.58 seconds. Initial focused run 31078 had 170 passes and one NameError: the legacy-init parity scenario used interactive(), which had moved to the wizard tests. The complete scenario now resides with that helper. Retry 95429 passed all 63 setup/wizard cases before the successful original-focus retry.

Combined architecture probe .tmp/p009-combined.yaml has no new dependency findings. It still reports ten missing contracts under old tooling/tests and examples, plus two incomplete Rust-root findings for examples/rust/crates/engine/src/lib.rs. The probe covers only selected maintained paths; source extensions select dependency analysis, not inventory coverage. Main checked tooling/worker/lint.yaml still does not enable architecture.

## Blockers

No operational blocker. Preserve behavior, exact executable oracles and the full maintained-tree scope. Root agentrig.yaml is strict; do not extract unsupported configuration packages or change the installed pin as a workaround.

## Next action

Finish the current setup/package VAC: inspect and stage only its paths, and commit through the normal gate. Refresh this snapshot with terminal evidence. Do not begin another implementation VAC before this one is accepted.

Then colocate remaining scaffold tests by actual owner: commands and jobs, gate, Git delivery, hooks/reminders and workflow composition. Feedback mixes evidence, resume and integration scenarios; split by behavior rather than copying helpers or moving the whole file under one misleading owner. Shared repository construction and revision observation have real gate/feedback/Git callers. Preserve exact oracles, decision markers, pytest discovery and file-relative fixture paths.

Remove evacuated old test directories after verifying only generated caches remain; remove obsolete command inputs and temporary external fixture exposure. Broaden contracts to maintained docs, resources and examples with explicit justified service/generated/third-party exclusions. Configure the example Rust crate root and other maintained language roots. Enable architecture in checked lint.yaml, verify both binaries and all four languages, complete full P009 acceptance and feature-merge, retaining the branch. A passing subset probe is not completion.
