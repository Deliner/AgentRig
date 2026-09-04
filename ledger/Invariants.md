# Invariants

Every row links observable behavior to its exact pytest oracle.

| ID | Invariant | Enforced by |
| --- | --- | --- |
| I001 | Every linked Python or shell decision application exists and carries its decision marker. | [test_decision_applications_require_markers](../tooling/tests/test_repo_policy.py) |
| I002 | Every invariant links an existing test carrying its invariant marker. | [test_invariant_links_require_marked_tests](../tooling/tests/test_repo_policy.py) |
| I003 | Committed decision identities, statements, details, and prior application links are append-only. | [test_committed_decisions_are_append_only](../tooling/tests/test_repo_policy.py) |
| I004 | File and directory soft limits warn while hard limits fail. | [test_size_thresholds_warn_then_fail](../tooling/tests/test_repo_policy.py) |
| I005 | Agent shell tools accept only one top-level invocation of a catalogued Just recipe. | [test_shell_guard_accepts_only_catalogued_just](../tooling/tests/test_command_workflow.py) |
| I006 | An edit checkpoint blocks another edit until commit and preserves unrelated pre-existing work. | [test_checkpoint_blocks_until_commit_and_preserves_unrelated_work](../tooling/tests/test_commit_checkpoint.py) |
| I007 | Complexity reminders use the configured attention and full-refresh intervals. | [test_complexity_schedule_is_loaded](../tooling/tests/test_complexity_reminder.py) |
