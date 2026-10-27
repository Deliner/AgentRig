# AgentRig 0.3.0

AgentRig is a structural linter for agent-native development, plus the environment that applies it for Codex and Claude Code. Source and documentation are licensed under MIT. Dependency terms and attribution are retained in THIRD_PARTY_LICENSES.json inside the binary archive.

Includes `agentrig` and the independently usable `agentrig-lint`. Capabilities include structural Rust/Python lint, strict YAML configuration, project initialization with an interactive wizard, doctor, memory and Git gates, feature-branch delivery and managed background jobs.

Download the archive for your platform and its `.sha256` file from this release, run `shasum -a 256 -c` on the checksum file, then extract it. Put both executables in a directory on PATH. Archives are built for Linux x86_64 (Ubuntu 24.04, glibc 2.39 or newer) and macOS arm64. Git and the project's configured tools are required. Read-only commands use bubblewrap on Linux and sandbox-exec on macOS.

Create a consumer with `agentrig init --root ./my-project --interactive`. See the repository's scaffold guide and independent Python/Rust examples for configuration and prerequisites. Configuration is YAML only. There is no upgrade path: a changed release is installed into a fresh project.

Development of AgentRig itself uses a separately installed verified Git revision. Candidate edits and build failures do not replace the active runtime. See tooling/distribution/README.md for explicit bootstrap and promotion.
