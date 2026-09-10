---
name: refactor-long-function
description: Reduce functions, methods or closures reported by function-lines while preserving their behavior.
---

# Separate a cohesive operation without changing its contract

Make the operation understandable through meaningful responsibilities while
resolving the reported size violation.

1. Read the operation and callers. Establish the observable contract, sequence,
   state dependencies, failure behavior and return behavior.
2. Identify a cohesive part whose purpose can be understood independently and
   whose inputs and effects remain explicit after extraction.
3. Choose the smallest extraction that resolves the finding while preserving
   those relationships. Keep interdependent steps together.
4. Verify the changed operation through its affected consumers.

Do not partition by position or equal size, compress lines, delete explanations
to reduce the count, or replace explicit dependencies with hidden state.
A smaller body is insufficient if understanding it now requires following
arbitrary fragments. Apply complexity-discipline; stop when the finding is
resolved and the contract and cohesion are preserved.

## Rule binding

Repair syntax before interpreting a failed parse. The measurement spans
signature through body and counts nonblank lines, including comments, nested
definitions and docstrings; decorators are excluded. Rerun the reported lint
check and relevant behavior checks. Do not raise thresholds merely to pass.
