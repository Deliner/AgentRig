# Invariants

Every row links observable behavior to its exact pytest oracle.

| ID | Invariant | Enforced by |
| --- | --- | --- |
| [I001](Invariants/001.md) | Every linked Rust, Python or shell decision application exists and carries its decision marker. | [test_decision_source_markers](../tooling/worker/src/scaffold/memory/tests/test_memory.py) |
| [I002](Invariants/002.md) | Every invariant links an existing test carrying its invariant marker. | [test_independent_project_delivery](../tooling/tests/native/scaffold/test_integration.py) |
| [I003](Invariants/003.md) | Committed decision identities, statements and details are immutable; current application links point to existing marked owners. | [test_committed_decisions_checked_against_staged_memory](../tooling/worker/src/scaffold/memory/tests/test_memory.py) |
| [I004](Invariants/004.md) | File and directory soft limits warn while hard limits fail. | [test_native_lint_thresholds_and_skills](../tooling/worker/src/lint/tests/test_lint.py) |
| [I005](Invariants/005.md) | Agent shell tools accept only one top-level invocation of a catalogued Just recipe. | [test_shell_guard_rejects_bypass](../tooling/tests/native/scaffold/test_hooks.py) |
| [I006](Invariants/006.md) | Failed staged checks reject commits; repeated edits and corrections remain available, and unrelated unstaged work is excluded. | [test_staged_commit_preserves_unstaged_work](../tooling/tests/native/scaffold/test_git.py) |
| [I007](Invariants/007.md) | Complexity reminders use the configured attention and full-refresh intervals. | [test_portable_reminder_schedule_retry_and_compaction](../tooling/tests/native/scaffold/test_reminders.py) |
| [I008](Invariants/008.md) | Project and Ledger are sibling roots, and every indexed Plan, Decision, and Invariant entry has its matching detail file. | [test_repository_layout_and_detail_contract](../tooling/worker/src/scaffold/memory/tests/test_memory.py) |
| [I009](Invariants/009.md) | The Plan accepts idle and paused delivery, allows at most one active feature, and rejects malformed contracts or invalid prerequisites. | [test_plan_delivery_contract](../tooling/worker/src/scaffold/memory/tests/test_plan.py) |
| [I010](Invariants/010.md) | Direct master commits fail; integration uses current master, preserves feature commits, and retains the merged feature reference. | [test_divergent_rebase_preserves_merge_history](../tooling/tests/native/scaffold/test_git.py) |
| [I011](Invariants/011.md) | State requires complete recovery sections while allowing stale recorded Git facts to be reconciled on resume. | [test_state_recovery_contract](../tooling/worker/src/scaffold/memory/tests/test_state.py) |
| [I012](Invariants/012.md) | Ledger edits receive matching skill guidance without a new edit denial, preserving existing guard and complexity decisions. | [test_ledger_edit_guidance](../tooling/tests/native/scaffold/test_hooks.py) |
| [I014](Invariants/014.md) | Invalid lint configuration blocks with an actionable repair skill. | [test_invalid_config_is_actionable](../tooling/worker/src/lint/tests/test_lint.py) |
| [I015](Invariants/015.md) | Supported language handlers distinguish named branch values from inline expressions and honor configured warning/block behavior with source locations. | [test_language_conditions](../tooling/worker/src/lint/languages/tests/test_language.py) |
| [I016](Invariants/016.md) | Configuration checking and lint reject unsupported language selections with an actionable error. | [test_config_rejects_unsupported_language](../tooling/worker/src/lint/config/tests/test_config_check.py) |
| [I017](Invariants/017.md) | Installed Rust/Python language defaults block unnamed conditions, functions above 40 nonblank lines and signatures above 4 inputs; boundary values pass. | [test_strict_defaults_block_delivery](../tooling/worker/src/scaffold/package/tests/test_lint_defaults.py) |
| [I018](Invariants/018.md) | Changed source or executable mode invalidates current check evidence, and a selected passing check never claims a full passing gate. | [test_evidence_tracks_content_and_partial_checks](../tooling/tests/native/scaffold/test_feedback.py) |
| [I019](Invariants/019.md) | Resume reports real merge and rebase conflicts without modifying repository contents. | [test_resume_observes_real_conflict](../tooling/tests/native/scaffold/test_feedback.py) |
| [I020](Invariants/020.md) | A new installation records correct shipped checksums, executable flags and ownership while retaining the baseline after user edits. | [test_installation_manifest_records_ownership](../tooling/worker/src/scaffold/package/tests/test_manifest.py) |
