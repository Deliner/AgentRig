# State

## Focus

Deliver active P009: complete checked architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Full acceptance and integration remain unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 2cc8c30

Installed runtime 9aee4ae remains pinned. No merge or rebase is in progress. Current unfinished VAC moves memory, plan and State tests into scaffold/memory/tests and extracts their existing shared memory builders into consumer.py. Gate and feedback consume that fixture directly. Exact configured oracle paths, invariant links and decision application links follow the moves; predicates, markers and scenario assertions remain unchanged.

## Progress

Expanded inventories, repair guidance, four-language dependency analysis, Rust binding and literal module-path handling are committed. Review, lint, delegate, upgrade, launcher and setup capability tests are colocated with their owners. Shared fixtures remain with their actual consumers' capabilities.

Latest accepted VAC 2cc8c30 separates review CLI verification, setup capability scenarios and shared scaffold consumer construction. Review resources belong to review/testing/consumer.py; Claude preference and registration helpers belong to setup/tests/harness.py. Scaffold's common project builder is scaffold/testing/consumer.py.

Current memory contracts describe test responsibilities and expose only consumer.py for actual external gate/feedback callers. Temporary public fixture entries in src and scaffold must be removed when remaining external consumers move inside their owners.

P010 is separately planned and pending for configuration hook deadlock. Do not implement it instead of P009 or modify voxel-rust.

## Verification

Commit 2cc8c30 passed normal gate 24312 (terminal 0): 94 Rust tests, 499 selected Python tests in 696.12 seconds, review tests, Clippy, formatting, memory and remaining configured checks. This is commit selection evidence, not full P009 acceptance.

Current memory VAC passes 137 memory/gate/feedback scenarios in 270.88 seconds (70587 terminal 0), mypy on 57 sources, Ruff, formatting, structural lint and memory check 35961. Pytest collects all 701 cases in 0.56 seconds. All 16 moved test-function ASTs match HEAD after the required repository-root depth correction.

Combined architecture probe .tmp/p009-combined.yaml has no new dependency findings. It still reports ten missing contracts under old tooling/tests and examples, plus two incomplete Rust-root findings for examples/rust/crates/engine/src/lib.rs. The probe covers only selected maintained paths; source extensions select dependency analysis, not inventory coverage. Main checked tooling/worker/lint.yaml still does not enable architecture.

## Blockers

No operational blocker. Preserve behavior, exact executable oracles and the full maintained-tree scope. Root agentrig.yaml is strict; do not extract unsupported configuration packages or change the installed pin as a workaround.

## Next action

Finish the current memory VAC: inspect and stage only its paths, and commit through the normal gate. Refresh this snapshot with terminal gate evidence. Do not begin another implementation VAC before this one is accepted.

Then colocate remaining scaffold tests by actual owner: commands and jobs, gate, Git delivery, hooks/reminders, setup/package and workflow composition. Feedback mixes evidence, resume and integration scenarios; split by behavior rather than copying helpers or moving the whole file under one misleading owner. Shared repository construction and revision observation have real gate/feedback/Git callers. Preserve exact oracles, decision markers, pytest discovery and file-relative fixture paths.

Remove evacuated old test directories after verifying only generated caches remain; remove obsolete command inputs and temporary external fixture exposure. Broaden contracts to maintained docs, resources and examples with explicit justified service/generated/third-party exclusions. Configure the example Rust crate root and other maintained language roots. Enable architecture in checked lint.yaml, verify both binaries and all four languages, complete full P009 acceptance and feature-merge, retaining the branch. A passing subset probe is not completion.
