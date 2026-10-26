# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26", including the agent's questions and the user's answers. The user
told the agent to start work. The order is: removal (D033), macOS port,
affected-only tests on feature merge, then layout, lint rules and tests.

## Workspace

Branch: feature/target-state

Revision: 10fbe31618730cc343a0af3d2868cb4d37f10dec

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins the
port commit and `just bootstrap` installed it under .cache/development. A stray
branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion.

## Progress

The removal under [D033](Decisions/033.md) is commit da71c68 and the macOS
port under [D034](Decisions/034.md) is commit 10fbe31. This VAC pins the
runtime to 10fbe31 and adds flock to macOS prerequisites and CI. Plan
P018–P023 predates this direction and is not reconciled yet.

## Verification

The installed runtime ran `check --staged` on this VAC on macOS: lint, memory,
rustfmt, clippy, ruff, mypy, typos, vulture and the selected config-check
tests passed in about 20 seconds. Before the port commit, the full pytest
suite gave 419 passed on macOS. Linux runtime behavior awaits CI.

## Blockers

None. The gate runs on macOS, so commits go through the pre-commit hook again.

## Next action

Verify agent hook registration for Claude Code and Codex in this repository on
macOS, then make feature-merge run only affected tests instead of the full
suite.
