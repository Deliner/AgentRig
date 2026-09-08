# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Full acceptance and integration remain unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: a371fd0

Installed runtime 9aee4ae remains pinned. Review test ownership and literal Rust module paths are committed as a371fd0. Current unfinished VAC moves 11 architecture Python test files into tooling/worker/src/lint/architecture/tests and the shared worker fixture into tooling/conftest.py. Imports, private VCS example path, D019 application link, mypy/Vulture inputs and affected test targets follow the moves. No commit gate is running.

## Progress

Expanded exact inventories, repair guidance, four-language dependency analysis and Rust binding fixes are committed. Module declarations retain access checks without manufacturing use cycles; actual imports, calls, types and reexports retain cycle checks.

Rust capabilities have directory contracts. Python tests import local modules by full paths; pytest retains its default import mode with the repository import root. Native Python test colocation and broader maintained-tree coverage are still pending.

Review tests now belong to their actual behavior owners. MCP scenario moved from run; YAML selection scenarios moved from VCS to configuration; snapshot scenarios moved from VCS to snapshot. Native/external repository builders are shared from vcs/tests with explicit public file boundaries. VCS no longer depends on configuration or snapshot. All 54 original relocated test names remain; the mixed external YAML/adapter scenario retains YAML assertions under its original name and adapter resolution under a new VCS test name.

P010 is separately planned and pending. Its recorded candidate reproduction confirms malformed configuration blocks diagnostic read and repair events with a misleading generic message. Do not implement it instead of P009 or touch voxel-rust.

## Verification

Latest Python ownership verification: common prepare/lint/explain helpers now live in lint/testing/consumer.py, with exact public boundaries for existing external test consumers. Architecture tests import this fixture instead of old test modules. Session 71401 exited 0: 224 affected architecture/scalar/configuration/explanation consumers passed in 17.64 seconds. Ruff passes, mypy passes 52 sources, memory check 83189 passed. Ruff-format found one trailing blank line in the new fixture; formatter corrected it. Combined Rust/Python probe .tmp/p009-combined.yaml has no new dependency/access/cycle findings: remaining findings are 12 preexisting missing contracts and two incomplete-root findings for the newly included Rust example. Full maintained-tree acceptance is still pending.


Commit retry 71837 exited 0 and created a371fd0: 73 Rust tests, 694 selected Python tests in 525.07 seconds, all review tests and every configured check passed. Initial formatting failure below is resolved.

Current architecture Python relocation: 149 tests passed in 10.94 seconds (37794 terminal 0); complete pytest collection reports 701 cases (5360 terminal 0); mypy passes all 51 sources. Exact new test contract and parent child responsibility added. Full Python architecture analysis has not yet been rerun with the new source paths.

Commit gate 44819 terminated with exit 1 at review-rustfmt after passing 73 Rust tests, 694 selected Python tests in 544.37 seconds, all review behavior tests and other configured checks. Formatting the main Cargo manifest had not formatted the separate review crate. Explicit cargo fmt for tooling/worker/review/Cargo.toml corrected the listed files; focused just check --only review-rustfmt --staged now passes. This is not a successful commit; normal retry remains required.

Latest temporary Rust architecture probe, just candidate lint --config .tmp/p009-self.yaml --json, exited 0 with [] after adding MCP and both snapshot test roots. This supersedes the earlier 24/244 findings. Scope covers configured Rust roots plus review fixtures, not full P009 maintained-tree acceptance.

Focused review checks passed: session 59927 had 14 execution + 1 MCP; 66365 had 6 YAML + 10 native VCS + 13 external VCS before snapshot relocation; final session 65255 had 8 native VCS + 12 external VCS + 2 native snapshot + 1 external snapshot. The only relocation compile error was a missing Path and unused hg import, both corrected. Clippy sessions 47772 and 60593 exited 0. Both native test_review.py consumers passed in 0.14 seconds. git diff --check passed; original relocated scenario-name audit found no missing names.

Literal Rust path support previously passed 25 source/resolver tests with compiler parity (8540) and 12 native module-wiring cases across both binaries (44462). Supports literal file-level external modules including raw unescaped strings and sibling paths in discovered sources; unsupported forms stay explicit. Source inventory does not assume modules lie below the crate root parent.

HEAD 9b903a2 normal commit gate 25008 passed: 696 selected Python tests in 661.42 seconds and all configured checks, Rust/review skipped. This proves that prior commit, not the current unfinished VAC. No full integration gate has run for P009.

## Blockers

No operational blocker. Preserve behavior, architectural permissions and the full maintained-tree scope. Root agentrig.yaml is strict and does not support setup configuration packages; do not extract packages into its root or update the installed pin as a workaround.

## Next action

Finish current architecture Python test relocation VAC through its normal commit gate. Shared lint CLI fixtures now have their own owner and all existing consumers point there. Combined source analysis confirms repaired enclosing boundaries. Include State, D019 application-link migration, command/check paths, moved conftest and 11 architecture test files, fixture consumers and exact contracts. The review VAC is already accepted. Continue remaining test ownership and full maintained-tree inventory after this VAC, including the example Rust crate root in final analysis configuration.

Then colocate native Python behavior tests with responsible features, updating exact check targets, invariant oracle bindings, decision application links, fixture discovery and __file__ resource paths. Read matching memory skills before those edits.

Broaden architecture coverage to maintained tests, docs and resources with explicit justified service/generated/third-party exclusions and exact responsibilities. Canonical skills live under tooling/worker/assets/skills; symlinks are skipped. Enable architecture in checked tooling/worker/lint.yaml, verify both binaries and all four languages, complete full P009 acceptance and integrate through feature-merge while retaining the branch. A clean Rust probe alone is not completion.
