# Invariants

Every row links observable behavior to its exact pytest oracle.

| ID | Invariant | Enforced by |
| --- | --- | --- |
| [I001](Invariants/001.md) | Every linked Python or shell decision application exists and carries its decision marker. | [test_decision_applications_require_markers](../tooling/tests/test_repo_policy.py) |
| [I002](Invariants/002.md) | Every invariant links an existing test carrying its invariant marker. | [test_invariant_links_require_marked_tests](../tooling/tests/test_repo_policy.py) |
| [I003](Invariants/003.md) | Committed decision identities, statements, details, and prior application links are append-only. | [test_committed_decisions_are_append_only](../tooling/tests/test_repo_policy.py) |
| [I004](Invariants/004.md) | File and directory soft limits warn while hard limits fail. | [test_native_lint_thresholds_and_skills](../tooling/tests/native/test_lint.py) |
| [I005](Invariants/005.md) | Agent shell tools accept only one top-level invocation of a catalogued Just recipe. | [test_shell_guard_accepts_only_catalogued_just](../tooling/tests/test_command_workflow.py) |
| [I006](Invariants/006.md) | Failed staged checks reject commits; repeated edits and corrections remain available, and unrelated unstaged work is excluded. | [test_vac_commit_boundary](../tooling/tests/test_commit_checkpoint.py) |
| [I007](Invariants/007.md) | Complexity reminders use the configured attention and full-refresh intervals. | [test_complexity_schedule_is_loaded](../tooling/tests/test_complexity_reminder.py) |
| [I008](Invariants/008.md) | Project and Ledger are sibling roots, and every indexed Plan, Decision, and Invariant entry has its matching detail file. | [test_worker_layout_and_detail_indexes](../tooling/tests/test_repo_policy.py) |
| [I009](Invariants/009.md) | The Plan accepts idle and paused delivery, allows at most one active feature, and rejects malformed contracts or invalid prerequisites. | [test_plan_delivery_contract](../tooling/tests/test_plan_policy.py) |
| [I010](Invariants/010.md) | Direct master commits fail; integration uses current master, preserves feature commits, and retains the merged feature reference. | [test_feature_branch_policy](../tooling/tests/test_branch_workflow.py) |
| [I011](Invariants/011.md) | State requires complete recovery sections while allowing stale recorded Git facts to be reconciled on resume. | [test_state_recovery_contract](../tooling/tests/test_state_policy.py) |
| [I012](Invariants/012.md) | Ledger edits receive matching skill guidance without a new edit denial, preserving existing guard and complexity decisions. | [test_ledger_edit_guidance](../tooling/tests/test_agent_context.py) |
| [I013](Invariants/013.md) | Native hooks preserve established guidance, guard behavior, and reminder transitions. | [test_native_hook_parity](../tooling/tests/native/test_hooks.py) |
| [I014](Invariants/014.md) | Invalid lint configuration blocks with an actionable repair skill. | [test_invalid_config_is_actionable](../tooling/tests/native/test_lint.py) |
| [I015](Invariants/015.md) | Supported language handlers distinguish named branch values from inline expressions and honor configured warning/block behavior with source locations. | [test_language_conditions](../tooling/tests/native/test_language.py) |
