# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26", including the agent's questions and the user's answers. The user
told the agent to start work. The order is: removal (D033), macOS port,
affected-only tests on feature merge, then layout, lint rules and tests.

## Workspace

Branch: feature/target-state

Revision: 17fd0916d8f8684117b1ed4a08fa7aca293c3551

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
3d91c85, installed by `just bootstrap` under .cache/development. A stray
branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion.

## Progress

Committed: removal under [D033](Decisions/033.md) (da71c68), macOS port under
[D034](Decisions/034.md) (10fbe31), runtime pin and macOS flock (6a06206),
Claude Code hook registration (4330b7c), affected-only merge tests under
[D035](Decisions/035.md) (8910f1f), its runtime pin (b4741c6), and the
line-width and forbidden-file rule kinds with condense-markdown under
[D036](Decisions/036.md) (09a0c97), the comment-reference kind with
replace-comment under [D037](Decisions/037.md) (0d1fe6d), and the runtime pin
to 0d1fe6d (d55e0a6), the D036 Markdown rules with condensed README and
AGENTS.md files (f5be8c9), and the D037 comments rule with its 42 findings
resolved (c79b35e), and [D038](Decisions/038.md) configuration modules in the
runtime (3d91c85) with the repository switched to them and 3d91c85 pinned
(17fd091). This VAC moves the repository Git hooks from .githooks to
.agentrig/hooks, the location init gives consumers, and registers that path in
core.hooksPath and bootstrap; the root then has room for the Just recipe
directory. That work (D039: root justfile importing just/*.just groups, init
template, decision) is stashed as "just recipe groups (D039)" and passed its
setup and delivery tests. Plan P018–P023 predates this direction and is not
reconciled.

## Verification

`just check` passed on 3d91c85 (68 Rust, 422 pytest, all static checks).
For this VAC, core.hooksPath reads .agentrig/hooks, `just lint` reports no
errors and memory-check passes; the commit gate runs from the new path.

## Blockers

The installed hook validates .agentrig/lint.yaml on every event, so rules
of a new kind can be configured only after the runtime that knows the kind is
pinned and installed. Otherwise the hook rejects every edit, including the
revert (B001). memory-check cannot run inside `just read`.

## Next action

Commit the hooks move through the hook, then pop the D039 stash, re-point its
.githooks references, verify and commit it. After that, the source move into
Project.
