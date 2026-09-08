# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance and integration remain unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: ce97ff7

Installed runtime 9aee4ae remains pinned. Rust architecture ownership and inventories are committed. Current VAC replaces 69 short local Python test imports with explicit module paths and aligns pytest/mypy package roots. No test function, oracle path or runtime behavior changes are intended.

## Progress

Expanded exact inventories, repair guidance, four-language dependency analysis and Rust binding fixes are committed. Module declarations retain access checks without manufacturing use cycles; real imports, calls, types and reexports retain cycle checks.

Rust capabilities now have exact directory contracts. Shared arguments, paths, resources, settings, receipts, installation state and saved-plan recovery have their own owners. Init and its wizard belong to setup. The standalone lint entry belongs to lint. Review MCP owns its unchanged argument schema. Gate orchestration lives with target selection; D004/D005/D015/D016 application links follow it.

Python tests now import uniquely identified local modules by full paths. Pytest uses the repository import root. Mypy explicit_package_bases resolves the observed duplicate short/full module identity. Its default pytest import mode is preserved. No local code was declared external and no analyzer feature was added to accommodate implicit pytest paths.

P010 is separately planned and pending after the user's configuration-hook deadlock report. The candidate reproduces denial of read and corrective edit events on malformed YAML; diagnostics hide the actual parsing cause. Do not implement P010 instead of P009 or touch voxel-rust.

## Verification

Review ownership commit caa4b63 passed gate 54205: 69 AgentRig Rust tests, 692 selected Python tests in 511.91 seconds, review crate tests and all configured checks.

Final Rust composition commit ce97ff7 passed gate 64164 (terminal 0): seven Rust tests, 252 selected Python tests in 426.87 seconds and all configured checks. Review tests skipped. Candidate build, Clippy, memory and eleven focused gate consumers passed before that commit.

Probe .tmp/p009-scaffold-composition.json exits 0 with no findings across the configured Rust source roots. This probe still has source-extension selection and does not prove full maintained-tree coverage.

Current explicit Python imports: all 698 native tests collect; 63 architecture, delegation MCP and State cases pass in 0.98 seconds (89072). Mypy passes all 51 configured sources after the reported duplicate-module correction. Ruff import sorting fixed seven findings; formatting left 48 files unchanged.

Probe .tmp/p009-python-qualified.json reports only 12 missing contracts and no unresolved local imports, compared with the prior 69 unresolved imports. Initial commit gate 90717 failed Ruff because local and exported configurations classified tooling imports differently. Explicit known-first-party tooling and the configured import sorter corrected 33 findings; the required staged Ruff retry passed. Normal commit retry is pending.

## Blockers

No current operational blocker. Preserve the full P009 scope, behavior, permissions and maintained-source coverage.

Live agentrig.yaml is strict and does not support setup configuration packages at its root. Do not retry root package extraction or change the installed development pin as a workaround. The root file currently has 500 nonblank lines; do not weaken thresholds.

## Next action

Finish the explicit Python import VAC through its required commit gate. Then colocate behavior tests with their actual feature/module owners, keeping shared test environment fixtures with their real owner and resolving measured coupling rather than widening permissions.

Remaining Rust review tests are under review/tests: Cargo can preserve their current target names with explicit relocated paths. Shared support/mod.rs and critic.py have consumers in execution/harness tests and native test_review.py. Inspect those consumers before moving them. Other tests already have explicit targets colocated with config, response and snapshot.

Native Python tests still live under tooling/tests/native. Moving them requires updating exact configured check targets, invariant oracles and decision application links, fixture discovery and __file__-relative resource paths. Read matching memory skills before updates. Preserve actual test collection and executable coverage.

Broaden architecture coverage to maintained tests, docs and resources with explicit justified generated/service/third-party exclusions and exact descriptions. Canonical skills live under tooling/worker/assets/skills; symlinks are skipped. Enable the rule in checked lint.yaml, verify both binaries and all four languages, complete full P009 acceptance and integrate through feature-merge while retaining the branch. A clean Rust probe alone is not completion.
