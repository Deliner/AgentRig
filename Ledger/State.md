# State

## Focus

On 2026-10-27 the user deleted unneeded Backlog cards and selected the rest
for execution: [Plan](Plan.md) orders B001 → B003 → B002 → B008 → B009 →
B014 as P018, P019, P021, P025, P026 and P027. P018 (configuration recovery
without blocking the agent, [B001](Backlog/001.md)) is active. References to
the removed review runner are assessed by the executor at acceptance
([D033](Decisions/033.md)).

## Workspace

Branch: feature/target-state

Revision: 79ff8be42062b798b203d8ceb631e4bdb15d5513

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
d64607e, installed by `just bootstrap` under .cache/development. A stray
branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion. Agent shells lack cargo and uv on PATH;
prefix `env PATH=/Users/admin/.cargo/bin:/Users/admin/.local/bin:...` inside
`just write --`.

## Progress

49d36db records the Backlog removal and the new Plan selection. The next
P018 VAC adds hook configuration recovery ([D045](Decisions/045.md)): with an
invalid configuration only configuration-source edits and `just config-check`
pass, every denial carries the full error and repair skill, and validation
errors name their module file. Consumer acceptance in real Codex and Claude
Code sessions (K7) is not done yet.

## Verification

The reproduction in a scratch consumer showed every event denied before the
change. After it, three new agent_hooks Rust tests pass and the same scratch
probe shows config edits and `just config-check` allowed, other calls denied
with the cause, for missing, corrupt, unknown-field, invalid-command,
invalid-check and broken-lint cases.

## Blockers

None. The installed hook validates .agentrig/lint.yaml with the pinned
runtime, so new lint settings need a pin first. memory-check cannot run
inside `just read`.

## Next action

Pin and bootstrap the recovery runtime, then run the K7 consumer scenario:
break a module in Codex and Claude Code consumers, let a real session repair
it with unrelated changes present, and confirm commit checks still apply.
