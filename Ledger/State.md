# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26", including the agent's questions and the user's answers. The user
told the agent to start work. The order is: removal (D033), macOS port,
affected-only tests on feature merge, then layout, lint rules and tests.

## Workspace

Branch: feature/target-state

Revision: da71c68874c16c8c190966aef89419763605fd2e

The machine is macOS (Darwin 25.5.0). The pinned runtime 9aee4ae predates the
port and the `git` configuration key, so the installed hooks and gate still
cannot run here. A stray branch feature/init-probe at a14f284 remains from an
install probe; guard-reference refuses its deletion.

## Progress

The removal VAC under [D033](Decisions/033.md) is committed as da71c68. The
macOS port under [D034](Decisions/034.md) is ready to commit: process groups
replace systemd scopes, job identity uses /proc on Linux and proc_pidinfo on
macOS, read-only commands use sandbox-exec on macOS, init validates its
preview through a canonical temporary path, and CI and release packaging
cover macOS arm64. The Rust analyzer now measures cfg branches over test and
target-platform predicates. Plan P018–P023 predates this direction and is not
reconciled yet.

## Verification

On the port content before commit: cargo fmt and clippy -D warnings are clean,
64 Rust tests pass, the Linux branches pass clippy for x86_64-unknown-linux-gnu
in an isolated crate, candidate lint reports no errors, config-check and
memory-check pass, ruff, mypy, typos and vulture pass, and pytest gives 419
passed on macOS with the default /var TMPDIR and no leaked job processes.
Linux runtime behavior is not yet exercised; CI will do that.

## Blockers

The user authorized commits without the pre-commit hook until the gate works
on macOS. feature-merge still needs the full gate.

## Next action

Commit the port, pin stable.txt to it, run `just bootstrap`, and verify the
installed hooks, `just check` and Claude Code and Codex hook registration on
macOS. Then make feature-merge run only affected tests.
