# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance is unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: 61ebbe6

Current VAC extends VCS directory discovery so empty visible directories participate in architectural inventory. Native Git/Mercurial use their own ignore commands; external adapters provide working-directories. File-derived directory-size measurements are preserved. Development pin is unchanged. Integration remains pending.

## Progress

Inventory, repair guidance, Rust binding analysis, delegation/environment/review ownership and declaration/use cycle correction are committed. Standard Rust module declarations remain subject to access checks but no longer manufacture use cycles. Calls, imports, type references and reexports retain cycle checks.

Shared artifact bytes, snapshots and diagnostic ownership are committed through 61ebbe6. The current discovery fix preserves tracked-file parents, skips metadata/symlink directories and applies lint exclusions. The external Mercurial example and protocol documentation include the new operation. Unsupported directory inventory is an explicit error without native fallback. Existing private gate fixtures supply their directory inventory.

## Verification

Commit 61ebbe6 passed all gates (92749 exited 0): 676 Python tests in 643.43 seconds and 147 Rust tests.

Before the fix, all six new backend/binary cases incorrectly returned success for a missing empty-directory contract (78868). After the fix, 25 inventory tests passed (73320), then 48 inventory/CLI/private-gate tests passed (96661) and three additional file-derived count regressions passed. Review Clippy, both Rust format checks, touched Python Ruff/format, structural lint and whitespace pass. Source probe still reports 34 cycles and 28 missing contracts, with no other findings. This probe covers Rust sources, not the complete maintained repository. No check process remains active; mandatory commit gate is next.

## Blockers

No current blocker. Preserve permissions and maintained-source coverage.

## Next action

Commit the VCS directory discovery VAC through the mandatory gate and repair any failures without weakening coverage. Continue remaining worker lint/scaffold ownership and measured dependency repairs. Complete inventory and dependency checks across maintained source/tests/docs/resources, including Python test import roots, and justify service/generated/third-party exclusions. Existing discovery skips symlinks; canonical skills live under tooling/worker/assets/skills rather than the .agents/skills alias. Enable expanded checked policy, verify full P009 acceptance and integrate with feature-merge, retaining the branch.
