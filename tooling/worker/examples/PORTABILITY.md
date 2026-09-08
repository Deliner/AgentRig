# Portable worker delivery verification

## AgentRig 0.3 acceptance

Verified on Linux on 2026-09-06. AgentRig uses strict YAML configuration,
local reusable packages, explicit overrides and one resource bundle for installed
project and isolated delegate environments. The supported production migration
is discipline-worker 0.2.0 to AgentRig 0.3.0. TOML is read only by explicit
migration and historical/recovery handling; external tools retain their formats.

The following checks cover the current environment plan. They inspect observable
behavior rather than treating generated configuration alone as integration proof.

| Requirement | Current evidence |
| --- | --- |
| Strict YAML across capabilities | Shared codec and [YAML tests](../review/src/config/yaml_tests.rs); native root/lint/delegate validation rejects legacy format, duplicate/unknown keys, wrong types and unsupported capabilities. |
| Reusable packages and deterministic resolution | [Composition tests](../src/composition/tests.rs) cover scoped imports, stable identities, explicit overrides, versions, cycles, named lists, transitive paths and provenance. Setup verifies configuration/resource digests before applying. |
| Complete configuration inspection | `test_inspection_validates_and_exposes_all_selected_configurations` in [capability tests](../src/scaffold/package/setup/tests/test_capabilities.py) verifies effective settings, nested provenance, overrides, credential references and unchanged target files. |
| Independent consumers and portable lint | `test_shared_package_prepares_independent_python_and_rust_consumers` in [integration tests](../src/scaffold/testing/test_delivery.py) installs external declarations with shared-package overrides and distinct service paths, deletes sources, runs installed gates, repeats setup and runs a separate copied linter's discovery/example/explain/check commands. |
| Preview, wizard and preservation | [Setup tests](../src/scaffold/package/setup/tests) verify preview/application correspondence, conflicts before writes, Python/Rust wizard equivalence, selected layouts/capabilities and cancellation without partial installation. |
| Selected project/delegate resources | [Capability tests](../src/scaffold/package/setup/tests/test_capabilities.py) cover shared skills/programs/hooks/MCP, explicit profile overrides, setup/update/rollback and source removal. [Bubblewrap tests](../src/delegate/sandbox/tests.rs) attempt forbidden writes, verify frozen resources and prove a sibling profile inherits no unselected resources. |
| Real frontend and MCP | The installed consumer below passes read, artifacts and code runs, reconnect, actual custom MCP calls, skill/hook values, checked patches and cleanup. A separate bare-profile/cancellation probe confirms resource non-inheritance and cancellation of an observed running command. |
| Migration and recovery | [Upgrade tests](../src/scaffold/upgrade/tests) use the real committed 0.2.0 baseline; verify edited settings/skills/memory, external review/delegate resources, executable modes, custom Git adapters, stale/conflicting plans, interrupted verification, apply and rollback. |
| Mandatory controls and failure outcomes | [Delegate tests](../src/delegate/run/tests/test_delegate_run.py), [code tests](../src/delegate/code/tests/test_delegate_code.py) and review integration tests cover deadlines, owner-scoped cancellation, concurrent runs, failed result/checks, protected inputs and cleanup failures. No package or hook substitutes for the runner's result validation. |

The migration baseline is commit a3e4b4f4d83538e21fecc7ed30393cdcf576ed30.
Its executable SHA-256 is
4f66a433097ac02c9be0f4f7ef828588fb4b24fb8530d420458819268c789b8a.
The local executable and receipt remain in `.tmp/agentrig-baseline-0.2.0` at the
repository root; automated tests reconstruct the same committed baseline.

Real delegate evidence uses Codex 0.153.4 and gpt-5.6-sol/high. The retained
`.tmp/agentrig-environment-kwjhy7x1/acceptance.json` identifies all three runs.
The original declaration/resource package was deleted before execution. Each
response matched the values supplied independently by its skill, SessionStart
hook and MCP service. The artifacts report retains actual MCP `tools/call`
traffic. Code checks passed and the retained patch passed `git apply --check`;
the original checkout remained unchanged. MCP reconnect reused the same run IDs.
All private runtime directories were absent after completion.

`cancellation-acceptance.json` in that directory records a second probe:
cancellation followed an observed `/tools/sleep 60` invocation; the bare profile
passed with an empty environment receipt; earlier reports and checkout remained
unchanged. Its private directories were also removed. The separate
`.tmp/agentrig-hooks-yrxoi0oz` probe verifies all four supported hook events and
blocking Stop repair. `.tmp/agentrig-installed-discovery-x3yq3jb2/result.json`
records native frontend discovery of an installed custom skill after source
removal. These are paid-model observations, separate from deterministic tests.

That P004 acceptance covered Linux, local packages and the Codex frontend. Ready
executables still require the documented host tools, model authentication and
working bubblewrap/systemd support. That delivery supplied no remote registry,
extra OS/frontend or automatic patch merge. Release publication is separate from local
feature integration. See [P004 delivery](../../../Ledger/Plan/004.md) and Git for
the final gate and integration state.

## P008 native skill discovery

Observed on Linux on 2026-09-07 with disposable consumers and isolated client homes:

| Client | Native observation |
| --- | --- |
| Codex 0.153.4 | App-server `skills/list` discovered a setup-installed custom skill under `.agents/skills` with no loading errors. |
| Claude Code 2.1.201, project | Initialization listed the setup-installed `.claude/skills` skill in both `skills` and `slash_commands`; the startup hook executed. |
| Claude Code 2.1.201, delegate settings | An empty `--setting-sources` hid a skill in the private `CLAUDE_CONFIG_DIR`. Selecting `user` discovered it. With the same file also supplied by `--settings`, its SessionStart hook ran once; a project settings hook did not run. |

Delegates select only that private user scope; the existing read-only mounts and
explicit MCP configuration still own isolation. Claude probes used a synthetic
key and an unreachable loopback endpoint and ended before any model response.
These discovery-only observations are separate from the real model acceptance
recorded below.

## P008 Codex model acceptance

Observed on Linux on 2026-09-07 with Codex 0.153.4 and gpt-5.6-sol/high,
using independently installed candidate consumers after deleting their source packages.
Read, artifact and code delegation passed as run-53lZke, run-1MJbBH and
run-PPoBtH. Responses used distinct values supplied by an installed skill,
SessionStart hook and an observed MCP tool call. Reconnection retained run IDs,
the code patch passed `git apply --check`, the consumer checkout was preserved,
and private execution directories were removed. Evidence is retained in
`.tmp/agentrig-environment-yqn1m9ri/acceptance.json` and its run artifacts.

Real review through the consumer's generated MCP registration returned a
validated PASS for run-pjyGqb on an exact committed revision. The checkout was
preserved and the private execution directory removed. Evidence is retained in
`.tmp/p008-review-8sup9idw`, including the request, MCP exchange and report.
These observations establish current Codex model execution; Claude acceptance
is recorded below.

Configuration-update tests separately exercise both clients' model/API changes
and removal, hooks and MCP changes, retained user settings and permissions,
explicit conflict resolution, repeat setup and exact rollback. They also cover
client switching and rejection of disabled hooks before plan writes.

## P008 Claude through OpenRouter

Observed on Linux on 2026-09-07 with Claude Code 2.1.201 and an externally
loaded OpenRouter key. Native connection used `ANTHROPIC_AUTH_TOKEN` and
`https://openrouter.ai/api`, following the
[provider integration](https://openrouter.ai/docs/cookbook/coding-agents/claude-code-integration).
The cheapest discovered Anthropic model, `anthropic/claude-3-haiku`, connected
but failed the full task contracts. Free `nvidia/nemotron-3-super-120b-a12b:free`
completed these independent consumers after their source packages were removed:

- Review run-o7px9A: generated MCP registration, validated PASS, preserved
  checkout and removed runtime directory; `.tmp/p008-review-c_sd5tdi`.
- Read run-rSxFAL: exact independent skill/hook/MCP values and cleanup;
  `.tmp/agentrig-environment-bhno66v1/read-acceptance.json`.
- Artifacts run-dsgPOK: exact values, retained actual MCP tools/call and startup
  output, preserved checkout and cleanup;
  `.tmp/agentrig-environment-m57roq1f/artifacts-acceptance.json`.
- Code run-nOqUpc used the inexpensive paid variant
  `nvidia/nemotron-3-super-120b-a12b`: exact file bytes, code check exit 0,
  applicable patch, unchanged original checkout and cleanup;
  `.tmp/agentrig-environment-rh6u9fa5/acceptance.json`. A prior missing-newline
  edit failed the same unchanged check; the final task specified the exact bytes.

Read/artifact consumers used `ENABLE_TOOL_SEARCH=false` and
`CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS=1` through existing profile environment
references. These native [gateway settings](https://code.claude.com/docs/en/env-vars)
are consumer choices, not global AgentRig defaults. Initial runs had malformed
API responses; combined code probes returned HTTP 404. Focused free code
run-FPUmYi then received HTTP 429, `free-models-per-day`. The paid variant resolved
that quota obstacle. The focused code consumer verified editing and patch checks;
the read/artifact consumers separately verified configured skills, hooks and MCP.
All probes terminated and retained their reports.

Actual provider spending was $0.16007502, with $1.39727778 remaining; free runs
did not increase usage. Claude's displayed gateway cost estimates differed from
actual billing. Accounting is retained in `.tmp/p008-openrouter-cost.json`.

## Historical worker delivery observation

Observed on Linux on 2026-09-05, using worker revision fe31ee6 and a separate
consumer at /tmp/worker-consumer-6dthj_ga/consumer. The consumer contained its own
source, tests, declaration, machine contract and prompt; it used the installed
worker binary. The worker source repository was not a critic mount.

## Observed sequence

1. Setup from the consumer declaration installed assets, registered hooks/MCP
   and passed dependency diagnostics. Source tests ran through the consumer gate.
2. The official Python MCP client called the generated worker_review connection
   with Codex gpt-5.6-luna/high. The committed doubling implementation received
   PASS in 93.5 seconds. The critic checked zero, negative and fractional values.
3. Repeated setup left consumer files unchanged. Changing review parallelism and
   memory, then repeating setup, preserved those changes and configuration comments.
4. A separately compiled adjacent-release fixture changed the worker package pin
   from 0.2.0 to 0.3.0, its explicit migration pair, a repair skill and a stock
   review prompt. Upgrade planned no conflicts, installed those changes and passed
   config-check, doctor and the consumer check gate.
5. Rollback restored 47 original files byte-for-byte with their permissions,
   including configuration and memory. JSON/Markdown review reports survived.
   The first comparison also observed Python creating an untracked __pycache__
   during the consumer test; generated cache additions are distinguished from
   changes to original files.

The 0.3.0 artifact in this historical observation was a verification fixture,
not a published release. At that time the shipped transition was 0.1.0 → 0.2.0.
Current AgentRig migration and its real baseline are documented above.

## Evidence and limits

Run ID: run-1MQ3Fy. Candidate: 418993f483fb5685a533cbfcde68ca1469f5f889.
Contract digest: 5283421672b9cba9d44f94308de6b105367332985f22fc81d71fd2cfbbebfa8b.
Persistent JSON report SHA-256:
a8f0343d0ec272de877acea46863c288f32392dabc2ced0bc23da40f0db08b24.

The critic observed that the host home, project Git metadata and planted host
skills/hooks were absent; its fresh Codex configuration did not contain the
planted host configuration. The response was validated, both reports were saved,
the run's runtime directory was absent afterward, and temporary authentication
was deleted. Native sandbox tests separately verify read-only mounts and the
actual worker Stop hook; model observations alone do not establish permissions.
See [review compatibility evidence](../review/COMPATIBILITY.md).

Automated native tests cover standalone worker/linter parity for Git and non-Git
consumers, no assessed-file writes, external repair guidance, capability/schema
errors, installation ownership and setup conflicts. A final regression reproduced
setup changing an existing Codex configuration from 0600 to 0644; setup now reuses
upgrade file-state handling and preserves its permissions. The setup tests pass
with the original 0600 permissions and retained TOML comments.

The temporary smoke scripts and logs are retained in the worker's ignored .tmp
area; transition.json and the reports remain in the external fixture. Automated
commit gates use deterministic fixtures rather than making paid model calls.
