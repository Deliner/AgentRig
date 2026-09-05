# State

## Focus

Implement the authorized upgrade MVP: ownership manifest; upgrade plan/apply/rollback; preserved settings and memory; visible conflicts; stale-plan checks; recovery journal and post-update verification. Support one explicit adjacent-version transition. P001 remains paused.

## Workspace

Branch: feature/scaffold-upgrades

Revision: acaa0b3

The installation receipt VAC is committed and passed its full staged gate (184 tests). The current uncommitted VAC prepares reviewable 0.1.0 to 0.2.0 plans. The implementation plan is .tmp/scaffold-upgrades-plan.md.

## Progress

The 0.2.0 runtime routes upgrade planning before strict project-version validation. It exports a local release executable through init, compares stock receipts or reconstructs the old baseline from its real installed binary, and records concrete diffs, conflicts and checksummed preimages/payloads. Project settings and memory are preserved; TOML runtime migration retains comments. Custom lint/reminder paths are included in reviewed file states. Apply, conflict resolution, recovery journal and rollback remain required work.

## Verification

Twelve focused upgrade/package tests passed. Upgrade tests build actual pinned predecessor sources, exercising installations both with and without receipts, local skill conflicts, exact runtime-only configuration edits, custom lint paths and unsupported releases. Strict lint reports no errors. Full staged verification is pending for this VAC.

## Blockers

None. Tests can rebuild predecessors from Git through WORKER_SOURCE_ROOT, including inside the exported-index gate. No fake release versions or vendored predecessor implementation are used.

## Next action

Commit the planner VAC through the normal gate. Then implement apply with explicit conflict resolutions and stale-plan verification, an interruption-safe technical journal, resumable apply and rollback, and post-update config-check/doctor/full check. The unused initial journal/replace sketches are in .tmp/upgrade-journal.rs and .tmp/upgrade-replace.rs. Verify interruption and failure behavior with real neighboring binaries, then complete integration before marking the goal achieved.
