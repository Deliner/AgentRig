# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26", including the agent's questions and the user's answers. The user
told the agent to start work. The order is: removal (D033), macOS port,
affected-only tests on feature merge, then layout, lint rules and tests.

## Workspace

Branch: feature/target-state

Revision: 0d1fe6d5033ab4e4870524338dfe003dad60f929

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
0d1fe6d, installed by `just bootstrap` under .cache/development. A stray
branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion.

## Progress

Committed: removal under [D033](Decisions/033.md) (da71c68), macOS port under
[D034](Decisions/034.md) (10fbe31), runtime pin and macOS flock (6a06206),
Claude Code hook registration (4330b7c), affected-only merge tests under
[D035](Decisions/035.md) (8910f1f), its runtime pin (b4741c6), and the
line-width and forbidden-file rule kinds with condense-markdown under
[D036](Decisions/036.md) (09a0c97), and the comment-reference kind with
replace-comment under [D037](Decisions/037.md) (0d1fe6d). This VAC pins the
runtime to 0d1fe6d so the hook accepts lint configuration using these kinds.
Neither D036 nor D037 rules are configured in this repository yet. Plan
P018–P023 predates this direction and is not reconciled yet.

## Verification

`just check` on 0d1fe6d passed: 68 Rust tests, 421 pytest tests and every
static check. Pin and `just bootstrap` ran in one command; hooks keep working.

## Blockers

The installed hook validates tooling/worker/lint.yaml on every event, so rules
of a new kind can be configured only after the runtime that knows the kind is
pinned and installed. Otherwise the hook rejects every edit, including the
revert (B001). memory-check cannot run inside `just read`.

## Next action

Configure the D036 Markdown rules and the D037 comment rule, condense README
and AGENTS.md files, and replace existing comments and docstrings.
