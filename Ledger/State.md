# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26". The user rejected Project as the top-level feature: first-level
features are the entries directly in Project, with two-word names. They asked
to apply only that and then review the code and project layout
([D042](Decisions/042.md)). The full B003 revision draft is not agreed.

## Workspace

Branch: feature/target-state

Revision: 6c302411a4de7924cee02038bf354561330da904

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
af4beb1, installed by `just bootstrap` under .cache/development. A stray
branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion. Agent shells lack cargo and uv on PATH;
prefix `env PATH=/Users/admin/.cargo/bin:/Users/admin/.local/bin:...`.

## Progress

Committed on this branch: removal [D033](Decisions/033.md), macOS port
[D034](Decisions/034.md), affected-only merge tests [D035](Decisions/035.md),
Markdown and comment rules [D036](Decisions/036.md)/[D037](Decisions/037.md),
.agentrig modules [D038](Decisions/038.md), Just groups
[D039](Decisions/039.md), Ledger/Project init defaults
[D040](Decisions/040.md), and the worker under Project
[D041](Decisions/041.md), pinned as af4beb1.

This VAC applies D042. Project holds agent_hooks, change_delivery (gate, git),
command_line, command_runner (with jobs), project_config (with settings),
project_memory (with check evidence), project_setup, structural_lint and
shared (arguments, artifacts, diagnostics, paths, skills, testing with
examples, vcs, yaml). pyproject.toml and uv.lock moved to tooling; util.rs and
Project/rust-toolchain.toml were removed. Plan P018–P023 reconciliation
remains open.

## Verification

The installed af4beb1 `agentrig check` passed on a scratch copy with the same
restructure: build, lint, memory, rustfmt, clippy, 68 Rust tests, ruff, mypy,
423 pytest, typos and vulture. In the repository, config-check and
lint-config-check pass; the commit gate verifies the final contents.

## Blockers

The installed hook validates .agentrig/lint.yaml on every event, so rules
of a new kind can be configured only after the runtime that knows the kind is
pinned and installed. Otherwise the hook rejects every edit, including the
revert (B001). memory-check cannot run inside `just read`.

## Next action

Show the user the new Project layout and wait for their review and the rest
of the feature scheme they recall; the B003 draft awaits agreement under
D032 before Backlog, Plan or further structure changes. Independent explicit
work meanwhile: faster tests (Python to Rust, coverage).
