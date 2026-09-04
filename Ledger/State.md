# State

## Focus

Authorized worker task: add language-specific lint handlers for named if conditions, function/method size and declared input counts. Product P001 remains unchanged.

## Workspace

Observed branch feature/language-aware-lint, based on master e6366c3. Implementation VAC 6c2ad30 contains the Rust/Python handlers, schema/defaults, repair skills, behavioral tests and D017/I015. This handoff changes only State. No unrelated changes were observed. Inspect Git before resuming.

## Progress

Read and applied the full complexity-discipline and relevant Ledger/configuration/skill guidance. Tree-sitter handlers share per-file analysis and report locations; numeric rules support warning-only or blocking thresholds, while named conditions use an explicit level. Defaults expose existing violations as warnings (40 function lines, 4 parameters). Simple Rust if let retains bindings; inline boolean expressions and let chains are reported. Exact scope and limitations are in tooling/worker/README.md.

## Verification

Focused native tests passed 34 cases, covering both grammars, thresholds, receivers, parser errors (including missing unnamed tokens), skill diagnostics, selectors and staged source/config isolation. Three new skills passed quick_validate.py. The full staged gate for 6c2ad30 passed: 133 pytest cases, rustfmt, Clippy and all existing Python/metadata checks. Integration remains the next operation; inspect Git for later progress.

## Blockers

None external. The first parser test exposed a missing-token traversal bug; it is corrected and the regression passes. Initial new-rule lint reported 282 condition, 17 function-size and 6 parameter warnings in the then-current tree. These are reported debt, not a claim of strict conformance. No repository-wide rewrite was requested; enforcement is configurable.

## Next action

If this handoff is uncommitted, commit it through the normal staged gate. Then run just feature-merge from the clean feature branch, retaining its reference. If Git shows the branch already integrated, this task is complete; follow the next authorized instruction.
