# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26", including the agent's questions and the user's answers. The user
told the agent to start work. The order is: removal (D033), macOS port,
affected-only tests on feature merge, then layout, lint rules and tests.

## Workspace

Branch: feature/target-state

Revision: 4330b7c9937c543354770a119cf8992cb4aaa6d1

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
10fbe31, installed by `just bootstrap` under .cache/development. A stray
branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion.

## Progress

Committed: removal under [D033](Decisions/033.md) (da71c68), macOS port under
[D034](Decisions/034.md) (10fbe31), runtime pin and macOS flock (6a06206),
Claude Code hook registration (4330b7c). This VAC implements
[D035](Decisions/035.md): feature-merge maps `git diff base HEAD` paths
through the configured affected groups instead of running the full suite;
`just check` still runs everything. Plan P018–P023 predates this direction and
is not reconciled yet.

## Verification

Candidate gate and git tests pass (43 passed), including merge selection of
affected targets and full fallback for unmapped paths. The installed runtime
(10fbe31) still runs the full suite at merge until stable.txt is repinned.

## Blockers

None. Commits go through the pre-commit hook. memory-check runs oracles that
record jobs, so it cannot run inside `just read`.

## Next action

Commit this VAC, repin stable.txt to it and bootstrap, then start the
restructure and the Markdown and comment lint rules.
