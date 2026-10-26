# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26", including the agent's questions and the user's answers. The user
told the agent to start work. The order is: removal (D033), macOS port,
affected-only tests on feature merge, then layout, lint rules and tests.

## Workspace

Branch: feature/target-state

Revision: c79b35e36ec3baf747349ff4c7e88287728baeb7

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
to 0d1fe6d (d55e0a6), the D036 Markdown rules with condensed README and
AGENTS.md files (f5be8c9), and the D037 comments rule with its 42 findings
resolved (c79b35e). This VAC implements [D038](Decisions/038.md) in the
candidate: configuration modules under `.agentrig/` read and written by the
runtime, fixtures, skills and guides. The repository itself still uses
agentrig.yaml, read by the installed 0d1fe6d; its lint command passes
`--config` because the candidate no longer reads agentrig.yaml. Plan
P018–P023 predates this direction and is not reconciled.

## Verification

For this VAC: 68 Rust tests and all 422 pytest tests passed against the
candidate before the final fixture refactor; `just lint` reports no errors and
memory-check passes. The commit gate reruns the affected tests.

## Blockers

The installed hook validates tooling/worker/lint.yaml on every event, so rules
of a new kind can be configured only after the runtime that knows the kind is
pinned and installed. Otherwise the hook rejects every edit, including the
revert (B001). memory-check cannot run inside `just read`.

## Next action

Pass `just check` on this commit, then in one command pin it, split
agentrig.yaml into `.agentrig/` modules, move tooling/worker/lint.yaml to
`.agentrig/lint.yaml` and run `just bootstrap`; update references and commit.
After that: the Just recipe directory, then the source move into Project.
