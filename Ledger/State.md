# State

## Focus

P171: entry-use. VAC 16 moves the job record, tool files, tool check findings, process samples and the settings
schema out of their entries in command_runner and project_config.

## Workspace

Branch: feature/entry-boundaries
Revision: 9ef0b542d9ed9a5989ec81a87fbcdfd38d3e7d05

## Progress

- jobs/record.rs holds Grouping and Record, which job.rs and storage.rs both use; job.rs holds Job, owner and now;
  Capture joins process.rs; mod.rs keeps job observation.
- pid_reuse_guard/sample.rs holds Resources and Sample; tool_registry gains tool_files.rs and finding.rs.
- settings_schema gains config.rs, checks.rs and config_files.rs; name joins validation.rs; mod.rs only offers them.
- D046 now marks and links the patch payload paths in agent_hooks/input.rs; D005 keeps the schema entry.

## Verification

- Candidate entry lint: entry-use 117 to 96, entry-bypass 0, file-cycle 0, architecture clean.
- The first gate refused a job.rs and storage.rs file cycle; record.rs breaks it.
- Clippy and rustfmt pass.

## Blockers

The affected map in .agentrig/checks.yaml still names cli_dispatch/mod.rs for the git, gate and launcher groups. A
proposal adding commands.rs and dispatch.rs waits in .cache/runtime/proposal for the user's `! just approve`.

## Next action

Commit VAC 16, then move the items of the shared vcs, diagnostics and source_parsers entries into named files.
