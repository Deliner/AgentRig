# Discipline Worker

A configurable Rust harness for agent feature delivery through verified atomic changes (VACs), with recovery state, durable decisions, executable invariants and a shared commit/merge gate.

Start with [repository instructions](AGENTS.md), then `just resume`. [worker.toml](worker.toml) owns paths, commands, checks, hook routes and oracle bindings. `just list` lists configured commands; `just --list` lists thin recipes. Run catalog commands with `just run NAME -- ARGS`.

- [Scaffold guide](tooling/worker/SCAFFOLD.md): installation, configuration, memory schemas, recovery and Git lifecycle.
- [Linter guide](tooling/worker/README.md): rule semantics, languages, selectors, severity and repair skills.
- [Independent examples](tooling/worker/examples/README.md): Python and Rust consumers and repeatable measurements.
- [Feature delivery skill](.agents/skills/execute-plan-feature/SKILL.md): VACs, evolving the plan, blockers and branch handoffs.

Product acceptance lives in [Plan](Ledger/Plan.md); [State](Ledger/State.md) records the current task and next action. Compare State with Git before following it. [Decisions](Ledger/Decisions.md) links current owners and preserves historical rationale; [Invariants](Ledger/Invariants.md) links executable behavior checks. Detailed historical decisions are evidence, not a second set of current instructions; the index identifies their successors.

The runtime lives in tooling/worker. Skills have one source in tooling/worker/assets/skills; .agents/skills is a symlink to it, and installed packages embed those same files. Git and agent registrations are thin adapters. The full gate runs against the actual staged tree at commit and the integration candidate before merge. It validates configured contracts and checks; semantic atomicity and acceptance adequacy remain the executing agent's responsibility.
