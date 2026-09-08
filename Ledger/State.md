# State

## Focus

Fix the new context-drift audit findings under the user's explicit maintenance instruction: truthful State freshness, visible reminder failures, compatible Claude transcript observations and comparable repeated reviews.

## Workspace

Branch: feature/context-drift-fixes

Revision: e272ab5

Started from clean master b3b2563, which includes the Backlog delivery and memory test-selection correction. Previous feature/workflow-quality-plan is retained at 36b7823. Installed runtime remains pinned to 9aee4ae; candidate checks exercise the fixes.

## Progress

Five Terra low auditors completed the read-only audit. Two new defects were reproduced: branch-only State was called current after HEAD changed, and unavailable reminder storage silently suppressed periodic guidance. Repeated review also lacks comparison of its prompt/scope inputs. Claude token-format compatibility was demonstrated only with a synthetic fixture and requires verification before claiming support.

Commit e272ab5 makes branch-only State unverified while retaining stale for observed branch/revision differences. Its Git/Mercurial regression exercises another commit on the same branch and a branch mismatch.

The current VAC reports reminder failures through additionalContext while preserving memory guidance and ordinary editing. Repairing the storage path restores the existing schedule. Disabled reminders do not initialize storage. Transcript observations now accept Claude assistant/message.usage alongside Codex token_count, counting input plus cache creation/read and excluding output tokens. The repeated-review fix remains required.

P010-P016 retain their existing pending outcomes and dependencies; this bounded maintenance does not claim their full acceptance. Backlog remains delivered. The detailed audit is a local ignored artifact at .cache/audits/2026-09-08-context-drift.md.

## Verification

State freshness passed 12 focused tests and the e272ab5 staged gate: seven native and 209 selected Python scenarios. All 40 focused hook scenarios passed in 10.01 seconds, including both transcript formats, retries, compaction and storage recovery. Installed Claude Code 2.1.201 against a temporary loopback API with a synthetic key produced a real assistant/message.usage record with input/cache-creation/cache-read counts 10/20/70; no model API spending. The sanitized observation is .cache/audits/claude-transcript-observation.json. The current commit gate and integration remain pending.

## Blockers

None. Mercurial is provided by the configured test environment, although it was absent from the ordinary shell PATH used by the audit reproduction.

## Next action

Inspect and commit reminder/Claude fixes through the affected gate. Then correct repeated-review input comparability, run its focused checks and integrate through the full gate. Reconcile this pre-commit snapshot with Git and check evidence on resumption.
