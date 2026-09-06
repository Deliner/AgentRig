# Review MCP

Linux review service and shared Rust `review-runner`. Configured Codex roles
review a committed Git candidate inside separate bubblewrap sandboxes. The
runner validates file responses and computes the report without model-based
aggregation. It does not edit the reviewed project or its workflow memory.

## Build and call

Requirements: Rust 1.98.1, Git, bubblewrap with user namespaces enabled, and
native Codex plus its adjacent `codex-code-mode-host`. The tested CLI is listed
in [COMPATIBILITY.md](COMPATIBILITY.md). Standard Linux runtime binaries,
libraries, resolver files and TLS certificates are mounted read-only.

From this repository:

```sh
just review config-check tooling/worker/review/config/review.yaml
just review run tooling/worker/review/config/review.yaml /absolute/request.json
```

The review engine is a worker library. The main `agentrig` executable
exposes it through `review config-check CONFIG`, `review run CONFIG REQUEST_JSON`
and `review mcp CONFIG`. The package also retains a `review-runner` CLI over the
same library. Build from the worker workspace:

```sh
cargo +1.98.1 build --release --locked --manifest-path tooling/worker/Cargo.toml
cargo +1.98.1 build --release --locked --manifest-path tooling/worker/review/Cargo.toml
```

Run `agentrig init --root CONSUMER --review true` to install review
resources and the review skill with the worker. Edit the installed configuration,
contracts and prompts for the consumer before requesting review. Run `agentrig setup --root CONSUMER` to register MCP and check dependencies.
Repeated setup preserves consumer settings and reports conflicting adapters.

Example request file:

```json
{
  "root": "/absolute/project",
  "base": "main",
  "candidate": "HEAD",
  "tool": "review_code",
  "previous_report": null
}
```

CLI stdout is the complete report JSON. A successfully produced report may
have PASS, FAIL or BLOCKED verdict; callers must inspect it, not infer review
success from process exit alone. Startup or report-persistence errors exit 2.

Provide an authentication-only directory through `CODEX_HOME` containing
Codex `auth.json`. Only that file is copied, with mode 0600, into each private
CLI home. Never put authentication in project files or review configuration.
`REVIEW_CODEX_BIN` can select the native executable; otherwise PATH lookup also
handles the npm wrapper. The host user config, skills and hooks are not copied.
Codex may create fresh service configuration in its private home.

## Configuration and contracts

Review settings use strict YAML exclusively. Duplicate keys, unknown fields,
incorrect scalar types, anchors, aliases, tags and merge keys are rejected.
TOML review settings must be explicitly migrated; there is no format fallback.

Start with [config/review.yaml](config/review.yaml). It defines named tools and
shared reviewer definitions; role counts and names are not hardcoded. A role
specifies its model, reasoning effort and prompt. Each tool selects roles and a
project configuration. Restart MCP after changing its tool configuration.

Each reviewer declares `frontend: codex`, its model, reasoning effort and
prompt. Codex is the implemented executor; another frontend fails configuration
validation before any critic starts. Existing configurations without `frontend`
select Codex. The resolved frontend is retained in the report configuration.

Resource paths are relative to the configuration that declares them. Project
visibility and normative-document globs are relative to the checkout root.
The supplied code/research profiles are examples: adapt their visible paths,
normative document paths, prompts and machine contracts to the actual project.
Their `docs/requirements.md` must exist in the candidate. Each normative glob
must match an allowed candidate file.

The machine contract has schema_version 1 and a nonempty `requirements` list.
Every requirement has `id`, `text`, nonempty `reviewers`, and `allow_na`.
IDs and assignments are unique; every selected role has assigned requirements.
Documents explain requirements; they do not replace this explicit assignment.
Configuration validation rejects unknown fields, unsupported versions, invalid
roles/globs/resources, missing prompts and inconsistent contracts before calls.

`parallelism` bounds simultaneous critics. `timeout_seconds` is one shared
execution deadline including queued roles; Git preparation and report writes
are outside the process timeout loop. `format_attempts` bounds Stop validation
attempts per role. Exhaustion remains a technical failure even if a later file
becomes valid. The counter and validator run in the parent process; the mounted
read-only hook can only request validation through its role's socket.

## Visibility and results

Only selected tracked regular files are exported from Git objects. The live
checkout, untracked files and .git are absent. Both sides of changed paths,
including deletes and rename endpoints, must fit visibility before model calls.
Selected symlinks/submodules and known credential/control paths are rejected.
Private-key markers are also rejected. This is not universal secret detection:
the configuration author must exclude sensitive content from the allowlist.

Critics receive the candidate snapshot, base diff, manifest, contract and schema.
`/project`, `/review-input` and `/review-bin` are read-only. `/work`, `/tmp` and
the separate CLI home are private to one role. No other critic or package source
directory is mounted. Network access is retained for Codex model requests.

The shared [response schema](schemas/response.json) and validator enforce exact
run/role/candidate/contract identities and assigned requirement coverage. FAIL
needs evidence, finding and minimal_fix. Use repository-relative `path:line`
evidence so repeated review can identify unchanged files. Observations are
strings and never affect the verdict. Missing/invalid responses, process errors,
timeouts and BLOCKED checks exclude PASS. No requirement result is inferred
from another role or a previous report.

A repeated call passes the prior persistent JSON report and retains its original
base. The runner adds the previous-candidate repair diff and previous findings.
Every assigned requirement is rechecked, including formerly closed checks;
no closed result is carried forward. Changed contracts require a new review
boundary. A new FAIL or BLOCKED in unchanged scope needs `late_finding: true` and a
nonempty `previous_omission`. Unresolved prior findings remain ordinary failures.

## Reports and MCP clients

The sample configuration keeps `runtime/` and `reports/` inside this package.
Every call creates a fresh runtime directory and persistent JSON/Markdown report
with its run ID. Reports retain the requested and resolved Git boundary, file
hashes, contract/configuration/prompts/schema and their digests, model settings,
exact response text, validation attempt counts and bounded CLI stderr/events. Technical
errors are recorded as BLOCKED; cleanup errors are separate report fields.

Normal execution saves reports, deletes runtime and verifies absence. If report
storage fails, the runner attempts an emergency report inside the retained
runtime and reports its path; it does not silently discard the only evidence.
A cleanup failure may leave runtime files and is explicitly reported.

MCP uses newline-delimited JSON-RPC over stdio, protocol 2025-11-25. AgentRig setup
registers the worker connection automatically. For explicit registration, use
`agentrig review mcp CONFIG --root PROJECT`; the standalone library CLI uses
`review-runner mcp CONFIG`. CONFIG is the absolute review configuration path. Each configured
tool accepts `root`, `base`, `candidate`, optional `previous_report`; the selected
tool supplies its own name. Results contain both text JSON and structuredContent.
Set the client's tool timeout above the runner timeout plus preparation/report
overhead (for the sample 900-second deadline, use at least 960 seconds). The
real-client smoke is documented in COMPATIBILITY.md; no background job API is
needed for that tested client. Calls are handled sequentially per stdio session,
with parallelism inside each review.

The worker installs [review-project](../assets/skills/review-project/SKILL.md)
when review is enabled at initialization. Run `agentrig setup` to configure its MCP connection. This adapter selects the tool
and Git boundary and returns the report to the caller without additional critics.
