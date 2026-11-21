# State

## Focus

P172 delivers B178 entry-bypass. VAC 11 routes every use of command_runner through its entry: jobs, sandbox
and tool_registry are declared privately, and the entry offers the job, sandbox and tool check API.

## Workspace

Branch: feature/entry-boundaries
Revision: 899d8cdf98a707731771f6ca69f6c7211f6061ad

## Progress

- jobs declares its parts privately and offers clean_jobs, execute, exit_code, streamed and print_report.
- process.rs gains streamed, so Git runs a VCS command through one owner call instead of execute and exit_code.
- tool_registry offers PlacementSource and check_placement; the unused jobs directory and cancel are removed.
- gate, git, cli_dispatch, project_memory and project_setup use command_runner only through its entry.

## Verification

- `just run rustfmt` clean, `just run clippy` clean.
- Candidate lint with the entry rules: entry-bypass 97 to 64, entry-use 152 to 153 from streamed using Capture,
  architecture clean.

## Blockers

None.

## Next action

Commit VAC 11, then route the uses of project_memory check_evidence and format through the memory entry.
