# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26", including the agent's questions and the user's answers. The user
told the agent to start work. The order is: removal (D033), macOS port,
affected-only tests on feature merge, then layout, lint rules and tests.

## Workspace

Branch: feature/target-state

Revision: 5ccd95265bb04cd5fd7cfbd05749c31dfd282979

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
3d91c85, installed by `just bootstrap` under .cache/development. A stray
branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion. Agent shells lack cargo and uv on PATH;
prefix `env PATH=/Users/admin/.cargo/bin:/Users/admin/.local/bin:...`.

## Progress

Committed on this branch: removal [D033](Decisions/033.md), macOS port
[D034](Decisions/034.md), affected-only merge tests [D035](Decisions/035.md),
Markdown and comment rules [D036](Decisions/036.md)/[D037](Decisions/037.md),
.agentrig modules [D038](Decisions/038.md) with 3d91c85 pinned (17fd091),
hooks under .agentrig/hooks (2a3b85c), Just groups [D039](Decisions/039.md)
(a6f79b6), Ledger/Project init defaults [D040](Decisions/040.md) (a580844)
and worker guides condensed into slice READMEs (5ccd952).

This VAC moves the crate, assets, examples and Python harness from tooling
into Project under [D041](Decisions/041.md), superseding D025's placement.
tooling keeps worker/build, worker/run and distribution. The launcher test
builds its candidate through tooling/worker/build instead of the Project
fixture, which the architecture rule keeps private, and the impact map maps
tooling/worker and launcher tests to test_launcher.py.
bootstrap.sh still archives tooling/worker from the pinned 3d91c85.
RELEASE.md placement and Plan P018–P023 reconciliation remain open.

## Verification

For this VAC: full pytest 422 passed with one missing-fixture error in
test_launcher.py; after the direct build call tooling/distribution/tests
passed (5). ruff, ruff-format, mypy, vulture and typos pass; `just lint`
reports no errors; candidate and installed memory-check, config-check and
lint-config-check pass. Rust checks run in the commit gate.

## Blockers

The installed hook validates .agentrig/lint.yaml on every event, so rules
of a new kind can be configured only after the runtime that knows the kind is
pinned and installed. Otherwise the hook rejects every edit, including the
revert (B001). memory-check cannot run inside `just read`.

## Next action

Commit the Project move through the hook. Then pin that revision in one
command with bootstrap.sh archiving Project and `just bootstrap`.
