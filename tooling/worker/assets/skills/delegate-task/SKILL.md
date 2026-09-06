---
name: delegate-task
description: Delegate an authorized bounded task to a configured worker profile and recover its validated result through MCP.
---

Read `capabilities.delegation.config` in worker.toml and the selected profile before
choosing an executor. Profiles define modes, allowed files, programs, skills, MCP
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

Use delegate_cancel to cancel the owned task. Restore the same WORKER_OWNER when
resuming from another session. A pending launch may remain stopping until its
launcher exits; keep the run_id and observe that run until termination. Result
queries recover terminal reports and retry incomplete cleanup.

The service currently supports read and artifacts modes. Project edits and their
integration remain the calling workflow's responsibility; a delegate result does
not itself authorize publication, merge or acceptance of a feature.
