# State

## Focus

Fix the new context-drift audit findings under the user's explicit maintenance instruction: truthful State freshness, visible reminder failures, compatible Claude transcript observations and comparable repeated reviews.

## Workspace

Branch: feature/context-drift-fixes

Revision: 72e2520

Started from clean master b3b2563, which includes the Backlog delivery and memory test-selection correction. Previous feature/workflow-quality-plan is retained at 36b7823. Installed runtime remains pinned to 9aee4ae; candidate checks exercise the fixes.

## Progress

Five Terra low auditors identified the maintenance scope: false branch-only State freshness, silently lost reminders, incompatible Claude transcript accounting and repeated reviews using changed criteria. Each issue now has a focused behavioral regression; Claude's actual record format was also observed with the installed client.

Commit e272ab5 makes branch-only State unverified while retaining stale for observed branch/revision differences. Its Git/Mercurial regression exercises another commit on the same branch and a branch mismatch.

Commit 72e2520 reports reminder failures through additionalContext while preserving memory guidance and ordinary editing. Repairing the storage path restores the existing schedule. Disabled reminders do not initialize storage. Transcript observations accept Claude assistant/message.usage alongside Codex token_count, counting input plus cache creation/read and excluding output tokens.

The current VAC rejects previous reports with different prompts, visible/contract path selections or normative file content before launching critics. Model/timeout changes and reordered equivalent path lists remain permitted. Existing late-finding and repair behavior is preserved. Repeated-review regressions now share continuation.rs beside their owner, with explicit Cargo registration and architecture inventory.

P010-P016 retain their existing pending outcomes and dependencies; this bounded maintenance does not claim their full acceptance. Backlog remains delivered. The detailed audit is a local ignored artifact at .cache/audits/2026-09-08-context-drift.md.

## Verification

State freshness passed 12 focused tests and the e272ab5 staged gate: seven native and 209 selected Python scenarios. Reminder fixes passed 40 focused scenarios and the 72e2520 gate: two native and 117 selected Python scenarios. Installed Claude Code 2.1.201 against a temporary loopback API with a synthetic key produced a real assistant/message.usage record with input/cache-creation/cache-read counts 10/20/70; no model API spending. The sanitized observation is .cache/audits/claude-transcript-observation.json.

Three new repeated-review regressions first failed on the old implementation with unexpected PASS. After correction, all seven continuation and eleven execution tests passed with real bubblewrap and deterministic critic fixtures. The initial commit gate caught the missing continuation.rs Rust root in lint.yaml; registering it resolved the incomplete analysis and the staged lint retry passed. The third commit gate and full integration remain pending.

## Blockers

None. Mercurial is provided by the configured test environment, although it was absent from the ordinary shell PATH used by the audit reproduction.

## Next action

Inspect and commit the repeated-review VAC through the affected gate, then run full feature integration and publish the verified refs. Preserve the retained branch and the independently pinned installed runtime. Reconcile this pre-commit snapshot with Git and check evidence on resumption.
