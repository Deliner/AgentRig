---
name: refactor-large-directory
description: Repair directory size or directory-architecture findings through cohesive responsibilities, directed dependencies and explicit public boundaries, preserving consumers and behavior.
---

# Repair directory architecture

Read the finding and the affected directories' `architecture.yaml` contracts,
including enclosing boundaries. Distinguish an oversized directory, a missing
description, a forbidden dependency, private access, a cycle, and incomplete
dependency analysis. A finding does not by itself require moving files.

For incomplete analysis, inspect the reported source and documented resolver
limits before treating the graph as complete. Do not mark an unresolved local
module external or exclude it merely to suppress the finding.

Let:

```text
E = the affected container or containers
C = its observable contract
I = its contained elements
D = actual dependencies among them and their consumers
```

When decomposition is needed, partition `I` into the smallest set of cohesive
groups such that each group has one independently meaningful responsibility
and dependencies between groups are directional.

Proceed in order:

1. Establish `C`, consumers, path constraints, tests, and configuration. Treat
   allow/deny declarations and public interfaces as architectural intent.
2. Map each element to its responsibility, dependencies, lifecycle, and reason
   to change. Trace the reported edge or cycle to its actual callers and targets.
3. Group elements that share one owner or invariant.
4. Create a subcontainer only for a group with an independent responsibility.
5. Repair the dependency at its owner. Use an existing public interface for
   private access; move misplaced responsibility or a genuinely shared contract
   when that resolves the dependency direction. Introduce an interface only when
   the actual relationship needs one. Move complete groups when required and
   update directly affected references.
6. Preserve existing paths through compatibility only when consumers require it.
7. Describe each scoped directory's responsibility and intended permissions in
   its contract. Rerun the reported lint command and affected behavior checks;
   verify callers, public access and dependency direction after the repair.

Do not choose the number or depth of groups in advance. Do not group by name,
position, representation, or count alone. Do not create generic buckets,
single-element layers, reciprocal dependencies, or unrelated renames.
Do not widen allow/public patterns, remove deny constraints, hide a cycle behind
reexports, or relax exclusions just to pass. Change architectural policy only
when the required responsibility or consumer contract actually changes.

Passing the numeric limit alone is not completion. The affected behavior and
consumer contracts must remain valid; the reported architectural violations
and applicable hard limits must be resolved through cohesive ownership.

If no defensible partition preserves `C`, report the conflict instead of creating
arbitrary structure or weakening the contract.
