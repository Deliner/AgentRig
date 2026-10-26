# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26", including the agent's questions and the user's answers. The user
told the agent to start work. The order is: removal (D033), macOS port,
affected-only tests on feature merge, then layout, lint rules and tests.

## Workspace

Branch: feature/target-state

Revision: af4beb113d035e0a46aad1450124448ef5397daa

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
af4beb1, installed by `just bootstrap` under .cache/development. A stray
branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion. Agent shells lack cargo and uv on PATH;
prefix `env PATH=/Users/admin/.cargo/bin:/Users/admin/.local/bin:...`.

## Progress

Committed on this branch: removal [D033](Decisions/033.md), macOS port
[D034](Decisions/034.md), affected-only merge tests [D035](Decisions/035.md),
Markdown and comment rules [D036](Decisions/036.md)/[D037](Decisions/037.md),
.agentrig modules [D038](Decisions/038.md) with 3d91c85 pinned (17fd091),
hooks under .agentrig/hooks (2a3b85c), Just groups [D039](Decisions/039.md)
(a6f79b6), Ledger/Project init defaults [D040](Decisions/040.md) (a580844),
worker guides condensed into slice READMEs (5ccd952) and the worker moved
into Project under [D041](Decisions/041.md), superseding D025's placement
(af4beb1).

tooling keeps worker/build, worker/run and distribution. This VAC pins
af4beb1 and makes bootstrap.sh archive Project. RELEASE.md placement and Plan
P018–P023 reconciliation remain open.

## Verification

`just check` passed on af4beb1 (68 Rust, 423 pytest, all static checks); its
commit gate passed with 355 selected tests including test_launcher.py.

## Blockers

The installed hook validates .agentrig/lint.yaml on every event, so rules
of a new kind can be configured only after the runtime that knows the kind is
pinned and installed. Otherwise the hook rejects every edit, including the
revert (B001). memory-check cannot run inside `just read`.

## Next action

Commit the pin. Then place RELEASE.md and decide whether this repository
installs its pinned runtime under .agentrig/bin like a consumer, replacing
tooling/worker/run; continue with nested features, two-word first-level names
and architecture roles from Requests.md.
