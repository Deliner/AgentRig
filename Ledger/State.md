# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26", including the agent's questions and the user's answers. The user
told the agent to start work. The order is: removal (D033), macOS port,
affected-only tests on feature merge, then layout, lint rules and tests.

## Workspace

Branch: feature/target-state

Revision: d55e0a63efbc2c1d3624e8568d079304f92c1efa

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
[D036](Decisions/036.md) (09a0c97), the comment-reference kind with
replace-comment under [D037](Decisions/037.md) (0d1fe6d), and the runtime pin
to 0d1fe6d (d55e0a6). This VAC configures the D036 markdown-size,
markdown-width and markdown-placement rules in tooling/worker/lint.yaml and
condenses AGENTS.md and the README files to fit. The D037 comment rule is not
configured yet. Plan P018–P023 predates this direction and is not reconciled.

## Verification

`just check` on 0d1fe6d passed: 68 Rust tests, 421 pytest tests and every
static check. For this VAC, `just lint` reports no errors and no Markdown
findings, and memory-check passes.

## Blockers

The installed hook validates tooling/worker/lint.yaml on every event, so rules
of a new kind can be configured only after the runtime that knows the kind is
pinned and installed. Otherwise the hook rejects every edit, including the
revert (B001). memory-check cannot run inside `just read`.

## Next action

Configure the D037 comment rule in tooling/worker/lint.yaml and replace or
delete the existing comments and docstrings it reports.
