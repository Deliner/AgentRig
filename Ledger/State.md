# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26", including the agent's questions and the user's answers. The user
told the agent to start work. The order is: removal (D033), macOS port,
affected-only tests on feature merge, then layout, lint rules and tests.

## Workspace

Branch: feature/target-state

Revision: 6a062069e58e9fbae4f4f00cc0fb17be6eefd204

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
10fbe31, installed by `just bootstrap` under .cache/development. A stray
branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion.

## Progress

Committed: removal under [D033](Decisions/033.md) (da71c68), macOS port under
[D034](Decisions/034.md) (10fbe31), runtime pin and macOS flock (6a06206).
This VAC registers Claude Code hooks in .claude/settings.json, matching the
Codex registration, and imports AGENTS.md through .claude/CLAUDE.md because
the root is at its 15-entry limit. Plan P018–P023 predates this direction and
is not reconciled yet.

## Verification

Simulated SessionStart, Bash, Edit and Codex exec_command events through the
installed hook gave resume context, shell denial, State skill guidance and
allowed `just read`. In this Claude Code session the hooks took effect at
once: raw shell was denied, the complexity reminder fired on Write, and
AGENTS.md loaded through the new import. `just read` denied writes outside its
temporary directory and network access. Candidate lint reports no errors and
memory-check passes.

## Blockers

None. Commits go through the pre-commit hook. memory-check runs oracles that
record jobs, so it cannot run inside `just read`.

## Next action

Make feature-merge run only the tests affected by the branch instead of the
full suite.
