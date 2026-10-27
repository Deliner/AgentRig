# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26". The user agreed and selected [B018](Backlog/018.md) on
2026-10-27: features without nested features declare an architecture type
(flat, layered, hexagonal, onion, clean, custom) with required parts and
inward dependencies ([D044](Decisions/044.md)), delivered as active
[P024](Plan/024.md). First-level features in Project ([D042](Decisions/042.md))
still await the user's review; the full B003 revision draft is not agreed.

## Workspace

Branch: feature/target-state

Revision: 179c65096072985b633067206a6f3f76d7a908f1

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
af4beb1, installed by `just bootstrap` under .cache/development. A stray
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
[D043](Decisions/043.md) (1dfde97) and the saved B018 card (179c650).

This VAC adds feature types to the candidate: optional
`architecture.feature_root`, the contract `type`, required parts and inward
part dependencies, with the rule guide and repair skill. Plan P018–P023
reconciliation remains open.

## Verification

39 architecture Rust tests and clippy pass. The candidate lint with a scratch
copy of lint.yaml setting `feature_root: Project` reports exactly 44 untyped
leaf features and nothing else; an unselected root is a configuration error.
The commit gate verifies the whole change.

## Blockers

The installed hook validates .agentrig/lint.yaml on every event, so
`feature_root` can be configured only after the runtime that knows it is
pinned and installed. Otherwise the hook rejects every edit, including the
revert (B001). memory-check cannot run inside `just read`.

## Next action

Pin this commit in tooling/distribution/stable.txt and run `just bootstrap`;
then set `feature_root: Project` in .agentrig/lint.yaml and give each feature
without nested features its type under B018 А6.
