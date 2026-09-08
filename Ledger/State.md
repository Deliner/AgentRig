# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Full acceptance and integration remain unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 9b903a2

Installed runtime 9aee4ae remains pinned. Current staged VAC colocates review behavior tests with execution, run, MCP, configuration, snapshot and VCS. Shared review consumer Fixture/critic.py lives in review/testing; native/external repository fixtures stay with VCS. The same VAC adds literal Rust module path resolution required by these shared fixtures. Initial commit gate failed only at review-rustfmt; formatting is corrected and the normal commit retry is next.

## Progress

Expanded exact inventories, repair guidance, four-language dependency analysis and Rust binding fixes are committed. Module declarations retain access checks without manufacturing use cycles; actual imports, calls, types and reexports retain cycle checks.

Rust capabilities have directory contracts. Python tests import local modules by full paths; pytest retains its default import mode with the repository import root. Native Python test colocation and broader maintained-tree coverage are still pending.

Review tests now belong to their actual behavior owners. MCP scenario moved from run; YAML selection scenarios moved from VCS to configuration; snapshot scenarios moved from VCS to snapshot. Native/external repository builders are shared from vcs/tests with explicit public file boundaries. VCS no longer depends on configuration or snapshot. All 54 original relocated test names remain; the mixed external YAML/adapter scenario retains YAML assertions under its original name and adapter resolution under a new VCS test name.

P010 is separately planned and pending. Its recorded candidate reproduction confirms malformed configuration blocks diagnostic read and repair events with a misleading generic message. Do not implement it instead of P009 or touch voxel-rust.

## Verification

Commit gate 44819 terminated with exit 1 at review-rustfmt after passing 73 Rust tests, 694 selected Python tests in 544.37 seconds, all review behavior tests and other configured checks. Formatting the main Cargo manifest had not formatted the separate review crate. Explicit cargo fmt for tooling/worker/review/Cargo.toml corrected the listed files; focused just check --only review-rustfmt --staged now passes. This is not a successful commit; normal retry remains required.

Latest temporary Rust architecture probe, just candidate lint --config .tmp/p009-self.yaml --json, exited 0 with [] after adding MCP and both snapshot test roots. This supersedes the earlier 24/244 findings. Scope covers configured Rust roots plus review fixtures, not full P009 maintained-tree acceptance.

Focused review checks passed: session 59927 had 14 execution + 1 MCP; 66365 had 6 YAML + 10 native VCS + 13 external VCS before snapshot relocation; final session 65255 had 8 native VCS + 12 external VCS + 2 native snapshot + 1 external snapshot. The only relocation compile error was a missing Path and unused hg import, both corrected. Clippy sessions 47772 and 60593 exited 0. Both native test_review.py consumers passed in 0.14 seconds. git diff --check passed; original relocated scenario-name audit found no missing names.

Literal Rust path support previously passed 25 source/resolver tests with compiler parity (8540) and 12 native module-wiring cases across both binaries (44462). Supports literal file-level external modules including raw unescaped strings and sibling paths in discovered sources; unsupported forms stay explicit. Source inventory does not assume modules lie below the crate root parent.

HEAD 9b903a2 normal commit gate 25008 passed: 696 selected Python tests in 661.42 seconds and all configured checks, Rust/review skipped. This proves that prior commit, not the current unfinished VAC. No full integration gate has run for P009.

## Blockers

No operational blocker. Preserve behavior, architectural permissions and the full maintained-tree scope. Root agentrig.yaml is strict and does not support setup configuration packages; do not extract packages into its root or update the installed pin as a workaround.

## Next action

Inspect and stage this cohesive review test ownership/literal-path VAC, run its normal commit hook and fix any reported findings. Keep mandatory affected commit checks and reserve full integration gate for merge. Update State with actual outcome. Do not restart a quiet live gate.

Then colocate native Python behavior tests with responsible features, updating exact check targets, invariant oracle bindings, decision application links, fixture discovery and __file__ resource paths. Read matching memory skills before those edits.

Broaden architecture coverage to maintained tests, docs and resources with explicit justified service/generated/third-party exclusions and exact responsibilities. Canonical skills live under tooling/worker/assets/skills; symlinks are skipped. Enable architecture in checked tooling/worker/lint.yaml, verify both binaries and all four languages, complete full P009 acceptance and integrate through feature-merge while retaining the branch. A clean Rust probe alone is not completion.
