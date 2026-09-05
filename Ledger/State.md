# State

## Focus

Deliver P002: native worker capabilities, separate configuration, declarative setup and shared update ownership. Apply complexity-discipline.

## Workspace

Branch: feature/worker-capabilities

Revision: 1cbbdeb

The ownership/acceptance VAC is committed. The current VAC moves review into tooling/worker and links its library into the worker executable.

## Progress

Review now lives in tooling/worker/review in the worker Cargo workspace with one lockfile. Main worker review commands and the retained review-runner CLI call the same library. A distinct review-hook entrypoint avoids collision with worker hooks. The launcher fingerprints embedded JSON and excludes temporary review/report trees. Just and current guides point at the new owner. No duplicate Project implementation remains.

## Verification

The 21 relocated Rust review tests passed. Two new native worker tests passed for a full simulated review through the worker binary, its actual sandboxed Stop hook, and configured MCP discovery. Native Just review config-check validates the supplied profiles. Strict lint has no errors. Full staged gate for this VAC remains pending.

## Blockers

None. Review resource installation, project capability configuration, worker setup and the standalone lint executable are still required. Existing lint --root/--config behavior already supports read-only external assessment.

## Next action

Commit the native-review VAC, then add the independent lint CLI over the same engine and implement capability-driven setup using package manifests. Preserve settings/memory and verify consumer setup, real MCP review, reconfiguration, upgrade and rollback before completing P002.
