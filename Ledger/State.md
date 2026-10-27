# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26". The user agreed and selected [B018](Backlog/018.md) on
2026-10-27: features without nested features declare an architecture type
with required parts and inward dependencies ([D044](Decisions/044.md)). This
VAC completes it as [P024](Archive/Plan/024.md). First-level features in
Project ([D042](Decisions/042.md)) still await the user's review; the full
B003 revision draft is not agreed.

## Workspace

Branch: feature/target-state

Revision: d4734a47e76b3b01ef181959e6b14b5a6eb902c7

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
[D043](Decisions/043.md), feature types in the candidate (d64607e) and its
pin (d4734a4).

This VAC sets `feature_root: Project`, types 36 leaf features flat and 3
custom (structural_lint/architecture, shared/testing, command_runner/jobs)
and archives P024. feature/target-state is not integrated into master. Plan
P018–P023 reconciliation remains open.

## Verification

The d64607e gate passed (45 Rust tests, 252 selected pytest, static checks),
including 39 architecture Rust tests for classification, parts and direction.
On this tree the candidate lint reports no errors, and the installed
lint-config-check and config-check pass. Criterion results are in P024.

## Blockers

None for current work. The installed hook validates .agentrig/lint.yaml with
the pinned runtime, so new lint settings need a pin first (B001).
memory-check cannot run inside `just read`.

## Next action

Report B018 to the user and ask whether to integrate feature/target-state
into master with `just feature-merge`; then await their review of the layout
and the rest of the feature scheme.
