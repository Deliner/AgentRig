# State

## Focus

Deliver active P009: complete architecture maps and feature/module ownership across maintained AgentRig source, tests, documentation and resources. Acceptance is unfinished.

## Workspace

Branch: feature/architecture-ownership

Revision: f17e53e

Current authorized maintenance VAC adds affected-path test groups to command checks. Staged checks use the index diff, unknown paths retain full commands, and ordinary/revision/integration checks remain full. The installed development pin and AgentRig test policy are not changed yet. P009 integration remains pending.

## Progress

Inventory, repair guidance, Rust binding analysis, delegation/environment/review ownership and declaration/use cycle correction are committed. Standard Rust module declarations remain subject to access checks but no longer manufacture use cycles. Calls, imports, type references and reexports retain cycle checks.

VCS empty-directory discovery is committed in f17e53e, including native ignore handling, private working-directories, explicit unsupported-operation errors and preserved file-derived size counts. The current maintenance adds optional Check.affected groups, literal target union, empty selection skips and selective evidence that cannot set full_gate_passed. Existing configurations omit the new field and retain full commands. CHECKS.md, AGENTS.md and the canonical execution skill explain configured commit tests versus full integration verification.

## Verification

Commit f17e53e passed all gates (74086 exited 0): 687 Python tests in 710.53 seconds and 147 Rust tests.

Current maintenance passed all-target compilation, Clippy, mypy, both Rust format checks, touched Python Ruff/format, structural lint and whitespace. Twelve new behavior/configuration cases pass, including edits, deletions, renames, unstaged isolation, full fallback, explicit skips and a real feature-merge invoking the full command. Eight existing gate/evidence/integration cases passed (97179). Fixture-only failures from report parsing, an unignored runtime directory, function size and action-map typing were corrected; final checks pass. The narrow canonical skill edit passed quick_validate. All observed check processes are terminal; the mandatory commit gate is next.

## Blockers

No current blocker. Preserve permissions and maintained-source coverage.

## Next action

Commit this selective-test runtime support through the mandatory gate. Then promote the accepted full commit in tooling/distribution/stable.txt, bootstrap it, and enable explicit affected groups for the expensive AgentRig test command in agentrig.yaml. Keep shared infrastructure and unknown paths on full fallback; map documentation/memory-only paths explicitly where no tests are needed, retain fast checks, and verify installed-hook selection plus full integration behavior. The user's selective-commit/full-merge request is the current maintenance priority; P009 remains the active unfinished goal.

Then continue worker lint/scaffold ownership and measured dependency repairs. Complete inventory and dependency checks across maintained source/tests/docs/resources and justify service/generated/third-party exclusions. The Python probe in .tmp/p009-python.yaml reports 69 unresolved local test imports and 12 missing contracts; assess explicit package imports before adding resolver modes. Existing discovery skips symlinks; canonical skills live under tooling/worker/assets/skills rather than the .agents/skills alias. Enable expanded checked policy, verify full P009 acceptance and integrate with feature-merge, retaining the branch.
