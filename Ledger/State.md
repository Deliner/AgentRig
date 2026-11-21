# State

## Focus

P171 and P172: entry-use and entry-bypass. VAC 15 moves the command table and dispatch of cli_dispatch and the hook
answers and edited paths of agent_hooks out of their entries.

## Workspace

Branch: feature/entry-boundaries
Revision: f8a0b8b8b3009722e9c2fcb6668735ed7f87ccb4

## Progress

- cli_dispatch/commands.rs holds the command table and argument validation; dispatch.rs, renamed from project.rs,
  runs every command; mod.rs only declares both, and main.rs uses them by their files.
- agent_hooks/responses.rs holds Event, context and deny; input.rs gains paths and target; mod.rs offers deny.
- D213, D218 and D226 link the command table in cli_dispatch/commands.rs.

## Verification

- Candidate entry lint: entry-use 143 to 117, entry-bypass 0, architecture clean.
- Clippy and rustfmt pass.

## Blockers

The affected map in .agentrig/checks.yaml still names cli_dispatch/mod.rs for the git, gate and launcher groups. A
proposal adding commands.rs and dispatch.rs waits in .cache/runtime/proposal for the user's `! just approve`.

## Next action

Commit VAC 15, then move the items of command_runner/jobs and tool_registry entries into named files.
