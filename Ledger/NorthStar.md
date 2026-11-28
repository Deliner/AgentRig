# North Star

AgentRig is a deterministic harness that lets AI agents (Claude Code, Codex) develop a project autonomously for long
without losing code quality or architecture; it ships handlers for Rust and Python projects. The agent chooses the
engineering, the rig owns the process: Plan → feature branch → verified atomic changes (VACs) → merge. Its hooks route
agent edits and commands, its gate blocks a failing commit or merge, and its protected files and runtime pin change
only with the user's approval; VAC boundaries and acceptance remain the agent's. A rig mechanism binds to a point every
project has, such as a Git event, an agent hook event or a configured path, takes project rules from the `.agentrig/`
modules and holds without the agent's cooperation; one that needs a project's domain is a project check (D234).
