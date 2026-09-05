---
name: refactor-large-file
description: Architecturally decompose an oversized source file when a size check warns or fails, preserving observable behavior and interfaces.
---

# Decompose a large file

Let:

```text
E = the oversized element
C = its observable contract
B = its behaviors and state
D = dependencies among them
```

Partition `B` into the smallest set of cohesive units such that each unit owns
one independently meaningful responsibility and dependencies between units are
directional.

Proceed in order:

1. Establish `C`, consumers, tests, behaviors, state, and dependencies.
2. Group elements that share one invariant, lifecycle, or reason to change.
3. Separate groups whose behavior can be owned and changed independently.
4. Assign every responsibility one authoritative owner.
5. Preserve `C`; retain a facade only when existing consumers require it.
6. Update only directly affected dependencies, consumers, and tests.
7. Verify preserved behavior and report the resulting ownership structure.

Do not choose the number of units in advance. Do not split by position, line
count, naming symmetry, or hypothetical reuse. Do not duplicate ownership,
create reciprocal dependencies, compress representation, or refactor unrelated
behavior.

Passing the numeric limit alone is not completion. Completion requires preserved
behavior, satisfied hard limits, cohesive ownership, and no remaining
independently meaningful responsibilities mixed in one unit.

If no cohesive extraction exists, report the conflict instead of creating an
arbitrary split or weakening the enforced contract.
