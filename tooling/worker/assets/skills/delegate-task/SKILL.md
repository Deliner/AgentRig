---
name: delegate-task
description: Delegate an authorized bounded task to a configured worker profile and recover its validated result through MCP.
---

Read `capabilities.delegation.config` in agentrig.yaml and the selected profile before
choosing an executor. Profiles define modes, allowed files, programs, skills, hooks, MCP
services and credential references. Use `just delegate config-check` to diagnose
configuration. Setup registers worker_delegation from these settings; authentication
remains separate.

Call delegate_start with `profile`, `task` and `contract.result_schema`. For a
committed snapshot include `revision`; uncommitted checkout edits are not included.
Explicit `inputs` map sandbox names to allowed project-relative files. In artifacts
mode, `contract.artifacts` maps required output paths to maximum byte sizes. Read
mode returns a JSON report and cannot declare artifacts. Use the published MCP
input schema for the exact request shape. Match the contract to the requested
result; do not add unrelated deliverables.

Retain run_id and the recorded job owner in the current workflow's recovery state.
Poll delegate_status or delegate_result for that run after a timeout or reconnect;
an observation timeout does not authorize a duplicate start. Inspect the top-level
outcome: PASS requires successful execution, a validated result and cleanup.
RUNNING and UNKNOWN are not success. Return the retained report, artifact paths
and hashes, and any execution or cleanup errors to the caller.
The returned environment receipt identifies captured program and skill bytes;
configured hooks run automatically and do not replace the runner's result checks.

Use delegate_cancel to cancel the owned task. Restore the same WORKER_OWNER when
resuming from another session. A pending launch may remain stopping until its
launcher exits; keep the run_id and observe that run until termination. Result
queries recover terminal reports and retry incomplete cleanup.

For code mode, supply a committed `revision` and `contract.changes` with
`write_paths` globs and a nonempty `checks` map. Each check maps a name to argv,
starting with a program name declared by the profile. The delegate edits an
isolated /project; checks see it read-only and must put build outputs in /work or
/tmp. Execution and checks share the profile timeout. Inspect the returned code
report, check results and retained change.patch; failed checks cannot yield PASS.

Apply a successful patch only through the calling workflow's existing Git
boundary and gates. The service does not modify the caller's checkout. A delegate
result does not itself authorize publication, merge or acceptance of a feature.
