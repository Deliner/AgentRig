# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26". The user agreed and selected [B018](Backlog/018.md) on
2026-10-27: features without nested features declare an architecture type
with required parts and inward dependencies ([D044](Decisions/044.md)),
completed as [P024](Archive/Plan/024.md). First-level features in
Project ([D042](Decisions/042.md)) still await the user's review; the full
B003 revision draft is not agreed.

## Workspace

Branch: feature/target-state

Revision: 02d3dde824937abf8b0534a30f7e7ed833f743dd

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
d64607e, installed by `just bootstrap` under .cache/development. A stray
branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion. Agent shells lack cargo and uv on PATH;
prefix `env PATH=/Users/admin/.cargo/bin:/Users/admin/.local/bin:...` inside
`just write --`.

## Progress

Committed on this branch: removal [D033](Decisions/033.md), macOS port
[D034](Decisions/034.md), affected-only merge tests [D035](Decisions/035.md),
Markdown and comment rules [D036](Decisions/036.md)/[D037](Decisions/037.md),
.agentrig modules [D038](Decisions/038.md), Just groups
[D039](Decisions/039.md), Ledger/Project init defaults
[D040](Decisions/040.md), the worker under Project
[D041](Decisions/041.md), first-level features in Project
[D042](Decisions/042.md), contracts without permissions
[D043](Decisions/043.md), feature types (d64607e), their pin (d4734a4) and
36 flat plus 3 custom leaf features in Project (45624b3). With the user's
approval feature/target-state was merged into master as 02d3dde and
fast-forwarded to it; the branch is retained. Plan P018–P023 reconciliation
remains open.

## Verification

feature-merge verified the integrated candidate with tests affected by the
whole branch: build, lint, memory, rustfmt, clippy, Rust tests, ruff, mypy,
341 pytest, typos and vulture passed. B018 criterion results are in P024.

## Blockers

None for current work. The installed hook validates .agentrig/lint.yaml with
the pinned runtime, so new lint settings need a pin first (B001).
memory-check cannot run inside `just read`.

## Next action

Await the user's review of the Project layout and the rest of the feature
scheme they recall; continue target-state work on feature/target-state.
