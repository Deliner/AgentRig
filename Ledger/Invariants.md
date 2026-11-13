# Invariants

Every row links observable behavior to its exact pytest or cargo oracle.

| ID | Invariant | Enforced by |
| --- | --- | --- |
| [I001](Invariants/001.md) | Every linked Rust, Python or shell decision application exists and carries its decision marker. | [decision_source_markers](../Project/project_memory/tests/validation.rs) |
| [I002](Invariants/002.md) | Every invariant links an existing test carrying its invariant marker. | [invariant_oracle_must_name_a_discovered_test_carrying_its_marker](../Project/project_memory/tests/oracles.rs) |
| [I003](Invariants/003.md) | Committed decision identities, statements and details are immutable; current application links point to existing marked owners. | [committed_decisions_checked_against_staged_memory](../Project/project_memory/tests/validation.rs) |
| [I004](Invariants/004.md) | File and directory soft limits warn while hard limits fail. | [native_lint_thresholds_and_skills](../Project/structural_lint/tests/diagnostics.rs) |
| [I005](Invariants/005.md) | Agent shell tools accept only one top-level invocation of a catalogued Just recipe. | [shell_guard_rejects_bypass](../Project/agent_hooks/tests/shell.rs) |
| [I006](Invariants/006.md) | Failed staged checks reject commits; repeated edits and corrections remain available, and unrelated unstaged work is excluded. | [staged_commit_preserves_unstaged_work](../Project/change_delivery/git/tests/guards.rs) |
| [I007](Invariants/007.md) | Complexity reminders use the configured attention and full-refresh intervals. | [portable_reminder_schedule_retry_and_compaction](../Project/agent_hooks/tests/reminders.rs) |
| [I008](Invariants/008.md) | Project and Ledger are sibling roots, and every indexed Plan, Decision, and Invariant entry has its matching detail file. | [repository_layout_and_detail_contract](../Project/project_memory/tests/validation.rs) |
| [I009](Invariants/009.md) | The Plan accepts idle and paused delivery, allows at most one active feature, and rejects malformed contracts or invalid prerequisites. | [plan_delivery_contract](../Project/project_memory/tests/plan.rs) |
| [I010](Invariants/010.md) | Direct master commits fail; integration uses current master, preserves feature commits, and retains the merged feature reference. | [divergent_rebase_preserves_merge_history](../Project/change_delivery/git/tests/integration.rs) |
| [I011](Invariants/011.md) | State requires complete recovery sections while allowing stale recorded Git facts to be reconciled on resume. | [state_recovery_contract](../Project/project_memory/tests/state.rs) |
| [I012](Invariants/012.md) | Ledger edits receive matching skill guidance without a new edit denial, preserving existing guard and complexity decisions. | [ledger_edit_guidance](../Project/agent_hooks/tests/memory.rs) |
| [I014](Invariants/014.md) | Invalid lint configuration blocks with an actionable repair skill. | [invalid_config_is_actionable](../Project/structural_lint/tests/diagnostics.rs) |
| [I015](Invariants/015.md) | Supported language handlers distinguish named branch values from inline expressions and honor configured warning/block behavior with source locations. | [language_conditions](../Project/structural_lint/tests/languages.rs) |
| [I016](Invariants/016.md) | Configuration checking and lint reject unsupported language selections with an actionable error. | [config_rejects_unsupported_language](../Project/structural_lint/tests/configuration.rs) |
| [I017](Invariants/017.md) | Installed Rust/Python language defaults block unnamed conditions, functions above 40 nonblank lines and signatures above 4 inputs; boundary values pass. | [strict_defaults_block_delivery](../Project/project_setup/tests/defaults.rs) |
| [I018](Invariants/018.md) | Changed source or executable mode invalidates current check evidence, and a selected passing check never claims a full passing gate. | [evidence_tracks_content_and_partial_checks](../Project/project_memory/check_evidence/tests/mod.rs) |
| [I019](Invariants/019.md) | Resume reports real merge and rebase conflicts without modifying repository contents. | [resume_observes_real_conflict](../Project/project_memory/tests/resume.rs) |
| [I020](Invariants/020.md) | The init delivery and this repository carry the same universal capabilities — lint rule kinds, skills, memory and structure routes, and memory/lint/tests/tools checks — excluding manifest worker-local checks. | [init_delivers_the_universal_capabilities_of_this_repository](../Project/project_setup/initialization/tests/mod.rs) |
| [I021](Invariants/021.md) | A Git error while the reference guard resolves or checks a feature-branch deletion fails the transaction; it never permits the deletion. | [reference_guard_rejects_deletion_when_the_tip_cannot_be_resolved](../Project/change_delivery/git/tests/guards.rs) |
