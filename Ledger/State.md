# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26", including the agent's questions and the user's answers. The user
told the agent to start work. The order is: removal (D033), macOS port,
affected-only tests on feature merge, then layout, lint rules and tests.

## Workspace

Branch: feature/target-state

Revision: b4741c642cca9bc591fad41f190b3bbd523ce27b

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
8910f1f, installed by `just bootstrap` under .cache/development. A stray
branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion.

## Progress

Committed: removal under [D033](Decisions/033.md) (da71c68), macOS port under
[D034](Decisions/034.md) (10fbe31), runtime pin and macOS flock (6a06206),
Claude Code hook registration (4330b7c), affected-only merge tests under
[D035](Decisions/035.md) (8910f1f) and its runtime pin (b4741c6). This VAC
implements the lint side of [D036](Decisions/036.md): line-width and
forbidden-file rule kinds and the condense-markdown skill. The repository
rules are not configured yet. Plan P018–P023 predates this direction and is
not reconciled yet.

## Verification

Candidate rustfmt, Clippy and 66 Rust tests pass, including the new
line-width and forbidden-file tests; lint and package pytest pass (220);
candidate lint has no errors and memory-check passes.

## Blockers

The installed hook validates tooling/worker/lint.yaml on every event, so rules
of a new kind can be configured only after the runtime that knows the kind is
pinned and installed. Otherwise the hook rejects every edit, including the
revert (B001). memory-check cannot run inside `just read`.

## Next action

Commit, pass `just check`, then pin and bootstrap in one command. Then add the
markdown-size, markdown-width and markdown-placement rules and condense README
and AGENTS.md files to fit.
