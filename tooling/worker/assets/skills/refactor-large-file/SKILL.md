---
name: refactor-large-file
description: Architecturally decompose an oversized source file when a size check warns or fails, preserving observable behavior and interfaces.
---

# Restore cohesive ownership within the reported boundary

Resolve excessive size by separating responsibilities that can be understood
and changed independently, while preserving the observable contract.

1. Establish the affected behaviors, state, consumers, tests and dependencies.
2. Group elements by the invariant, lifecycle or reason to change they share.
   Distinguish ownership of behavior from use of another owner's capability.
3. Select the smallest set of cohesive units that resolves the finding. Give
   each responsibility one authoritative owner and keep dependencies directional.
4. Preserve the contract; retain a facade only when existing consumers require
   it. Update only directly affected references, consumers and tests.
5. Verify the preserved behavior and explain the resulting ownership.

Do not choose a unit count in advance or split by position, line count, naming
symmetry or hypothetical reuse. Do not duplicate ownership, create reciprocal
dependencies, compress representation or refactor unrelated behavior.

A lower count alone is not completion. The reported hard limits must be met
with cohesive ownership and preserved consumer behavior. Stop there; another
possible decomposition is not a reason to keep splitting. If no defensible
partition preserves the contract, report the conflict rather than inventing
structure or weakening the requirement. Apply complexity-discipline and rerun
the reported lint check and affected behavior checks.
