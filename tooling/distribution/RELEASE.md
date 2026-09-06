# AgentRig 0.3.0

First public Linux x86_64 release of AgentRig, a configurable environment generator and runtime for agent-native development. Source and documentation are licensed under MIT. Dependency terms and attribution are retained in THIRD_PARTY_LICENSES.json inside the binary archive.

Includes `agentrig` and the independently usable `agentrig-lint`. Capabilities include strict YAML configuration and reusable packages, setup preview and installation, structural Rust/Python lint, memory and Git gates, isolated review MCP, configured delegation and managed process cleanup. Custom skills, hooks, programs and MCP services are selected by configuration.

Download the archive and its `.sha256` file from this release, run `sha256sum -c agentrig-0.3.0-linux-x86_64.tar.gz.sha256`, then extract it. Put both executables in a directory on PATH. The release build uses Ubuntu 24.04; use a compatible Linux x86_64 system with glibc 2.39 or newer. Git and the project's configured tools are required. Read-only commands and model sandboxes require bubblewrap. Managed background work and delegation require a working systemd user manager. Review and delegation currently support the native Codex frontend; credentials are provided separately.

Create a consumer with `agentrig init --root ./my-project --interactive`, or use `agentrig setup --config /path/to/project.yaml --root ./existing-project --preview` followed by the same setup without `--preview`. The existing directory must exist for setup. See the repository's scaffold guide and independent Python/Rust examples for configuration and prerequisites.

Configuration is YAML only; no runtime TOML fallback exists. The supported release migration is explicitly 0.2.0 to 0.3.0: use the new executable's `upgrade plan /path/to/new/agentrig --root PROJECT`, review conflicts in the emitted plan, then `upgrade apply PLAN --root PROJECT`. Keep the new executable available for recovery; `upgrade rollback --root PROJECT` restores verified preimages while refusing to overwrite later edits. Same-version configuration updates use `upgrade plan --config CONFIG_YAML`. This release does not claim arbitrary version-to-version migration.

Development of AgentRig itself uses a separately installed verified Git revision. Candidate edits and build failures do not replace the active runtime. See tooling/distribution/README.md for explicit bootstrap and promotion.
