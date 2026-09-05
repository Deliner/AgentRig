# Portable worker delivery verification

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

The 0.3.0 artifact was a verification fixture, not a published release or a new
production migration matrix. The shipped transition remains the explicit
0.1.0 → 0.2.0 migration. A subsequent release owns its next transition.

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
