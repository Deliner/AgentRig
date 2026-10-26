# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26", including the agent's questions and the user's answers. The user
told the agent to start work. The order is: removal (D033), macOS port,
affected-only tests on feature merge, then layout, lint rules and tests.

## Workspace

Branch: feature/target-state

Revision: 8910f1f24033bbf2273872c282af9c7eb1b3c4a2

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
8910f1f, installed by `just bootstrap` under .cache/development. A stray
branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion.

## Progress

Committed: removal under [D033](Decisions/033.md) (da71c68), macOS port under
[D034](Decisions/034.md) (10fbe31), runtime pin and macOS flock (6a06206),
Claude Code hook registration (4330b7c), and affected-only merge tests under
[D035](Decisions/035.md) (8910f1f). This VAC pins the runtime to 8910f1f, so
`just feature-merge` selects tests for the whole branch. Plan P018–P023
predates this direction and is not reconciled yet.

## Verification

`just check` on 8910f1f passed: 64 Rust tests, 419 pytest tests and every
static check. `just bootstrap` installed 8910f1f.

## Blockers

None. Commits go through the pre-commit hook. memory-check runs oracles that
record jobs, so it cannot run inside `just read`.

## Next action

Commit the pin, then add the Markdown size, width and placement rules with a
condensing skill, and the comment rule allowing only decision and invariant
references.
