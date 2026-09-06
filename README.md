# AgentRig

AgentRig assembles reusable YAML configuration, skills, hooks and MCP services into portable environments for agent-native software development. The Rust worker supplies verified atomic changes (VACs), recovery state, durable decisions, executable invariants, structural lint, isolated review and configured delegates. Deterministic access boundaries and checks support development quality; model judgments remain probabilistic.

The worker is the product. Reusable capabilities belong under tooling/worker; Project holds consumer examples. Projects select capabilities through agentrig.yaml and separate lint/review configuration. Use `agentrig setup` to prepare a consumer from its declaration; `agentrig-lint` also runs independently against external projects. See [consumer setup and configuration](tooling/worker/SCAFFOLD.md).

Download the Linux x86_64 binaries from [GitHub releases](https://github.com/Deliner/AgentRig/releases). Verify the accompanying SHA-256 file, extract the archive and put `agentrig` and `agentrig-lint` on PATH. The binary release targets Ubuntu 24.04 or compatible glibc 2.39+ systems. Install Git, Just and your project's tools; read-only commands require bubblewrap, while background jobs and delegation require a systemd user manager. Model capabilities additionally require the native Codex CLI and separately provided authentication.

Create an environment with `agentrig init --root ./my-project --interactive`. For an existing directory and a reusable YAML declaration, inspect `agentrig setup --config /absolute/path/project.yaml --root ./my-project --preview`, then repeat without `--preview` to install. Run the installed binary's `config-check` and `doctor` from the consumer. Consumer projects use the shipped executable and their own checks; they do not need this repository's Rust build or test suite.

This project is [MIT licensed](LICENSE). The binary archive includes dependency licenses and attribution. See [release and migration notes](tooling/distribution/RELEASE.md).

To develop AgentRig, start with [repository instructions](AGENTS.md), run `just bootstrap` once, then `just resume`. The bootstrap installs a verified development revision; `just candidate ...` separately builds the product being edited. See [development and distribution](tooling/distribution/README.md). [agentrig.yaml](agentrig.yaml) owns paths, commands, checks, hook routes and oracle bindings. `just list` lists configured commands; `just --list` lists thin recipes. Run catalog commands with `just run NAME -- ARGS`.

- [Scaffold guide](tooling/worker/SCAFFOLD.md): installation, configuration, memory schemas, recovery and Git lifecycle.
- [Review guide](tooling/worker/review/README.md): configured isolated review through the worker.
- [Linter guide](tooling/worker/README.md): rule semantics, languages, selectors, severity and repair skills.
- [Independent examples](tooling/worker/examples/README.md): Python and Rust consumers and repeatable measurements.
- [Feature delivery skill](tooling/worker/assets/skills/execute-plan-feature/SKILL.md): VACs, evolving the plan, blockers and branch handoffs.

Product acceptance lives in [Plan](Ledger/Plan.md); [State](Ledger/State.md) records the current task and next action. Compare State with Git before following it. [Decisions](Ledger/Decisions.md) links current owners and preserves historical rationale; [Invariants](Ledger/Invariants.md) links executable behavior checks. Detailed historical decisions are evidence, not a second set of current instructions; the index identifies their successors.

The runtime lives in tooling/worker. Skills have one source in tooling/worker/assets/skills; .agents/skills is a symlink to it, and installed packages embed those same files. Git and agent registrations are thin adapters. The full gate runs against the actual staged tree at commit and the integration candidate before merge. It validates configured contracts and checks; semantic atomicity and acceptance adequacy remain the executing agent's responsibility.
