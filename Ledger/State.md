# State

## Focus

On 2026-10-28 the user asked for two things: completed Backlog cards must move
to the archive, and after each merge the agent must update the development
pin and run bootstrap itself, recording that rule in the instructions. The
selected Backlog set B001, B002, B003, B008, B009, B014 and B018 is complete
(P018–P027 in [Archive/Plan.md](Archive/Plan.md)); the [Plan](Plan.md) is
empty.

## Workspace

Branch: feature/backlog-archive

Revision: 7c4a03f6d272050102a5f0737bcc52cb38a5ae28

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
937d3f0, installed by bootstrap under .cache/development. Change the pin and
run bootstrap in one shell command: a pin without its install makes every
hook fail. A stray branch feature/init-probe at a14f284 remains from an
install probe; guard-reference refuses its deletion. `just read`/`just
write` run only the programs in .agentrig/commands.yaml. Agent shells lack
cargo on PATH, so commits run through `just scratch -- 'export
PATH=/Users/admin/.cargo/bin:...; git commit ...'`.

## Progress

[D050](Decisions/050.md): the seven completed cards moved to
[Archive/Backlog.md](Archive/Backlog.md) with links updated; Backlog is empty.
memory-check validates the archived Backlog and rejects IDs present in both;
edit-plan, edit-backlog, AGENTS.md and the shipped instructions describe the
rotation. The pin rule and the pin update itself are not done yet.

## Verification

On the uncommitted change the candidate passed Clippy, memory-check, lint
and 163 Python tests (project_memory, project_setup, agent_hooks). All
relative Ledger links resolve except three pre-existing links to the removed
tooling/worker/examples/PORTABILITY.md in archived P002, P004 and P008.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Commit this VAC, record the pin rule in AGENTS.md and the distribution guide,
pin the branch head and bootstrap it, commit the pin, run `just
feature-merge` and report to the user.
