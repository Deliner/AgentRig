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

Revision: 5a14d47da67970b34e608b0272d005affe7cabea

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
579474d (D045 recovery), installed by `just bootstrap` under
.cache/development. A stray branch feature/init-probe at a14f284 remains from
an install probe; guard-reference refuses its deletion. Agent shells lack
cargo and uv on PATH; prefix `PATH=/Users/admin/.cargo/bin:...` inside
`just write --`.

## Progress

579474d added hook configuration recovery ([D045](Decisions/045.md)). The
K7 Codex session exposed two gaps: apply_patch text in `tool_input.command`
was not read, so the permitted repair edit was denied; and Codex reads only
through shell, so it repaired blind and lost the test command. The current
VAC ([D046](Decisions/046.md)) reads that payload key and allows `cat` of
configuration sources and the repair skill during recovery.

## Verification

agent_hooks Rust tests (5) and test_hooks.py (29, including the Codex patch
shape) pass on the working tree. K7 in scratch consumers: Claude Code (haiku,
D045 candidate) and Codex (gpt-5.6-luna, D046 candidate, hooks trusted via
`--dangerously-bypass-hook-trust`) each saw the denial, read the skill and
config, fixed `read.accepts_args`, passed `just config-check`, created
Project/hello.txt and ran `just list`; Codex restored commands.yaml exactly,
and the untracked Project/notes.txt survived. guard-commit blocked a commit
while the consumer configuration was broken.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Commit the D046 VAC, pin and bootstrap it, record K1–K8 evidence in P018 and
archive it, then start P019 (B003): move `globs`/`skill` out of
structural_lint::config into shared support.
