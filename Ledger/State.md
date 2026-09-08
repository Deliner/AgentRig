# State

## Focus

Deliver active P009: complete checked architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Full acceptance and integration remain unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: ab32505

Installed runtime 9aee4ae remains pinned. No merge or rebase is in progress. Current unfinished VAC moves test_hooks.py and test_reminders.py unchanged into hooks/tests. Exact I005/I007/I012 targets, invariant links and decision application links follow their owners. Mypy and Vulture include hooks explicitly; the new architecture contract permits the existing scaffold project fixture.

## Progress

Expanded inventories, repair guidance, four-language dependency analysis, Rust binding and literal module-path handling are committed. Review, lint, delegate, upgrade, launcher and setup capability tests are colocated with their owners. Shared fixtures remain with their actual consumers' capabilities.

Latest accepted VAC ab32505 colocates setup/package behavior: client settings, layout, resource imports, declarative setup and wizard scenarios belong to setup; doctor, manifest and installed lint defaults belong to package. Their shared declaration/delegated_project helpers live in setup/tests/consumer.py. Previous 0d77816 colocated memory, plan and State tests.

Temporary public fixture entries remain in src/scaffold/package for actual external gate/feedback callers. Remove unnecessary outer exposure when remaining consumers move inside their owners. Hook tests continue to consume scaffold/testing/consumer.py through its actual public boundary.

P010 is separately planned and pending for configuration hook deadlock. Do not implement it instead of P009 or modify voxel-rust.

## Verification

Commit ab32505 passed normal gate 63045 (terminal 0): 94 Rust tests, 418 selected Python tests in 457.67 seconds, review tests and all remaining checks. Earlier 0d77816 passed gate 82431 with 418 Python tests. These are commit selections, not full P009 acceptance.

Current hooks VAC passes 35 scenarios in 1.21 seconds (17502 terminal 0), memory check 54658, Ruff, formatting and mypy on 63 sources. Initial mypy checked only 61 files because its explicit input list did not include the relocated tests; hooks was then added to both mypy and Vulture inputs and mypy rerun successfully. The two test files are unchanged moves; fixture paths and all scenario bodies remain intact.

Combined architecture probe .tmp/p009-combined.yaml has no new dependency findings. It still reports ten missing contracts under old tooling/tests and examples, plus two incomplete Rust-root findings for examples/rust/crates/engine/src/lib.rs. The probe covers only selected maintained paths; source extensions select dependency analysis, not inventory coverage. Main checked tooling/worker/lint.yaml still does not enable architecture.

## Blockers

No operational blocker. Preserve behavior, exact executable oracles and the full maintained-tree scope. Root agentrig.yaml is strict; do not extract unsupported configuration packages or change the installed pin as a workaround.

## Next action

Finish the current hooks VAC: inspect and stage only its paths, and commit through the normal gate. Refresh this snapshot with terminal evidence. Do not begin another implementation VAC before this one is accepted.

Then colocate remaining scaffold tests by actual owner: commands and jobs, gate, Git delivery and workflow composition. Feedback mixes evidence, resume and integration scenarios; split by behavior rather than copying helpers or moving the whole file under one misleading owner. Shared repository construction and revision observation have real gate/feedback/Git callers. Preserve exact oracles, decision markers, pytest discovery and file-relative fixture paths.

Remove evacuated old test directories after verifying only generated caches remain; remove obsolete command inputs and temporary external fixture exposure. Broaden contracts to maintained docs, resources and examples with explicit justified service/generated/third-party exclusions. Configure the example Rust crate root and other maintained language roots. Enable architecture in checked lint.yaml, verify both binaries and all four languages, complete full P009 acceptance and feature-merge, retaining the branch. A passing subset probe is not completion.
