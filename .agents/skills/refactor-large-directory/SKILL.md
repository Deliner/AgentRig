---
name: refactor-large-directory
description: Architecturally decompose an oversized directory when an item-count check warns or fails, preserving observable paths, dependencies, and behavior.
---

# Decompose a large directory

Let:

```text
E = the oversized container
C = its observable contract
I = its contained elements
D = dependencies among them
```

Partition `I` into the smallest set of cohesive groups such that each group has
one independently meaningful responsibility and dependencies between groups are
directional.

Proceed in order:

1. Establish `C`, consumers, path constraints, tests, and configuration.
2. Map each element to its responsibility, dependencies, lifecycle, and reason to change.
3. Group elements that share one owner or invariant.
4. Create a subcontainer only for a group with an independent responsibility.
5. Move the complete group and update only directly affected references.
6. Preserve existing paths through compatibility only when consumers require it.
7. Verify behavior, references, and the resulting ownership structure.

Do not choose the number or depth of groups in advance. Do not group by name,
position, representation, or count alone. Do not create generic buckets,
single-element layers, reciprocal dependencies, or unrelated renames.

Passing the numeric limit alone is not completion. Completion requires preserved
contracts, satisfied hard limits, cohesive ownership, and no remaining
independently meaningful groups mixed in one container.

If no defensible partition preserves `C`, report the conflict instead of creating
arbitrary structure or weakening the contract.
