---
name: reduce-branching
description: Reduce the decision points counted by branch-budget while preserving behavior.
---

# Spend the branch budget on one decision per function

Make each function carry about one decision's worth of branching. The budget
counts decision points, not lines: resolve a finding by removing or relocating
decisions, not by scattering the same ones across helpers that each still decide.

1. Read the function and establish every case it distinguishes, the condition
   that selects each, the order of effects and the result on every path.
2. Replace an `if`/`else if` ladder over one value with a single `match` on the
   real case, so the choice is stated once instead of re-tested on each rung.
3. Hoist a guard that handles a case and leaves into an early return, so its
   condition stops competing with the main decision.
4. Move a cohesive group of branches that names its own responsibility onto its
   owner — a method per variant, or a helper that owns that sub-decision — so the
   caller spends one branch reaching it.
5. Verify the changed function through its callers and affected behavior checks.

A `match` or `case` counts once however many arms it has: a flat multi-way
dispatch is one decision, so prefer it over a chain of `if`/`else if` that re-tests
the same value. Do not merge unrelated conditions or drop a case merely to lower
the count.

## Rule binding

The measurement counts the decision points in one function: each `if`, else-if
and ternary condition, and each `match` or `case` as one multi-way decision
regardless of arms; nested function and closure bodies are measured on their own.
Rerun the reported lint check and relevant behavior checks. Do not raise the
threshold merely to pass. Apply complexity-discipline; stop when the finding is
resolved and behavior is preserved.
