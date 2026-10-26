# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26", including the agent's questions and the user's answers. The user
told the agent to start work. The order is: removal (D033), macOS port,
affected-only tests on feature merge, then layout, lint rules and tests.

## Workspace

Branch: feature/target-state

Revision: 0fadd080932508723bca9f1dd73df9e70b17568d

The machine is macOS (Darwin 25.5.0). The pinned runtime 9aee4ae was
bootstrapped with a canonical TMPDIR, because the /var symlink breaks init
path checks. A stray branch feature/init-probe at a14f284 remains from an
install probe; guard-reference refuses its deletion.

## Progress

The removal VAC under [D033](Decisions/033.md) is ready to commit: review and
its MCP, delegation and its MCP, upgrade with receipt and recovery, setup,
preview, reconcile and package composition, Mercurial and external VCS, and
JS/TS/TSX analysis are gone. The VCS layer, YAML codec and artifacts moved
into tooling/worker/src; the branch policy key is `git`. I020 is retired.
Plan P018–P023 predates this direction and is not reconciled yet.

## Verification

On the removal content before commit: cargo clippy -D warnings and fmt are
clean, 62 Rust library tests pass, candidate lint over the repository reports
no errors, config-check passes, ruff, mypy, typos and vulture pass. pytest
against the candidate gives 376 passed, 38 failed, 6 errors; every failure
comes from /proc job identity, systemd scopes or the bwrap/doctor sandbox.

## Blockers

The staged gate and memory-check cannot run on macOS until the port. The user
authorized commits without the pre-commit hook until the gate works on macOS,
after build, Rust tests, clippy and applicable pytest. feature-merge still
needs the full gate.

## Next action

Port jobs to macOS: proc_pidinfo identity instead of /proc, process groups
instead of systemd scopes, sandbox-exec for read-only commands, canonical
init paths; then repin and bootstrap the runtime and run the gate.
