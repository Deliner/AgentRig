---
name: repair-verification
description: Repair a failed worker gate stage while preserving its actual contract and unrelated changes.
---

# Repair a gate failure

- Read the named stage and its original diagnostic. Fix the current VAC; do not disable a check, weaken a test, or raise thresholds merely to pass.
- For repo-policy, apply edit-plan, edit-decisions, edit-invariants, or edit-state according to the reported Ledger file. For command-policy, reconcile justfile recipes, What/Why comments, catalog entries, and dependencies.
- For Ruff/rustfmt formatting, format the affected files. For Ruff, mypy, Clippy, or Vulture findings, fix the reported implementation/type/ownership issue and preserve behavior.
- For pytest or Rust tests, reproduce the named failure and inspect expected versus observed behavior. Correct the implementation or an obsolete expectation only under the current contract; add a regression check when needed.
- For typos, fix the text or document a legitimate project term. Keep literals whose exact value is a contract.
- Rerun the focused failing check, then commit through the full staged gate. Record unresolved failures and the next recovery step in State before a handoff.
