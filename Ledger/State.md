# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26", including the agent's questions and the user's answers. The user
told the agent to start work. The order is: removal (D033), macOS port,
affected-only tests on feature merge, then layout, lint rules and tests.

## Workspace

Branch: feature/target-state

Revision: 09a0c977b2134382a3156bdd9edb2070566e2576

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
8910f1f, installed by `just bootstrap` under .cache/development. A stray
branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion.

## Progress

Committed: removal under [D033](Decisions/033.md) (da71c68), macOS port under
[D034](Decisions/034.md) (10fbe31), runtime pin and macOS flock (6a06206),
Claude Code hook registration (4330b7c), affected-only merge tests under
[D035](Decisions/035.md) (8910f1f), its runtime pin (b4741c6), and the
line-width and forbidden-file rule kinds with condense-markdown under
[D036](Decisions/036.md) (09a0c97). This VAC adds the comment-reference kind
and replace-comment skill under [D037](Decisions/037.md); consumer defaults
include it. Neither D036 nor D037 rules are configured in this repository
yet. Plan P018–P023 predates this direction and is not reconciled yet.

## Verification

Candidate rustfmt, Clippy and 68 Rust tests pass, including Rust and Python
comment measurement tests; lint and scaffold pytest pass (375), including
consumer comment defaults; candidate lint has no errors; memory-check passes.

## Blockers

The installed hook validates tooling/worker/lint.yaml on every event, so rules
of a new kind can be configured only after the runtime that knows the kind is
pinned and installed. Otherwise the hook rejects every edit, including the
revert (B001). memory-check cannot run inside `just read`.

## Next action

Commit, pass `just check`, then pin and bootstrap in one command. Then
configure the D036 Markdown rules and the D037 comment rule, condense README
and AGENTS.md files, and replace existing comments and docstrings.
