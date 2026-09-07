---
name: refactor-large-directory
description: Repair directory size, architecture inventory and dependency findings through feature ownership, cohesive modules and public boundaries, preserving consumers and behavior.
---

# Repair directory architecture

Read the finding and the affected directories' `architecture.yaml` contracts,
including enclosing boundaries. Distinguish an oversized directory, a missing
description or inventory entry, a forbidden dependency, private access, a cycle, and incomplete
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
3. Group elements by the feature or module that owns their behavior and reason
   to change. Keep feature-owned implementation, data, adapters and behavior
   tests together. A feature owns its scenario, not every capability it calls:
   a renderer used by several features remains with its shared owner, reached
   through its public interface. Compose the project from modules and cohesive
   features without imposing a fixed number of directory levels.
4. Create a subcontainer only for a group with an independent responsibility.
5. Repair the dependency at its owner. Use an existing public interface for
   private access; move misplaced responsibility or a genuinely shared contract
   when that resolves the dependency direction. Introduce an interface only when
   the actual relationship needs one. Move complete groups when required and
   update directly affected references.
6. Preserve existing paths through compatibility only when consumers require it.
7. Give each scoped directory one line of shared purpose. Register its files
   individually with what belongs in each, and describe child directories in
   the separate directories block. Each child has its own contract. Repair
   missing or stale entries against the actual intended ownership; do not
   automatically bless misplaced files by describing their current location.
   Preserve intended permissions. Rerun lint and affected behavior checks;
   verify callers, public access and dependency direction after the repair.

Do not choose the number or depth of groups in advance. Do not group by name,
position, representation, or count alone. Do not create generic buckets,
single-element layers, reciprocal dependencies, or unrelated renames. Do not
create a generic shared bucket merely because several files need a home.
Separate build packages need a real dependency, infrastructure, build or test
boundary. Onion layers and interfaces need actual responsibilities; directory
navigation alone does not justify them.
Do not widen allow/public patterns, remove deny constraints, hide a cycle behind
reexports, or relax exclusions just to pass. Change architectural policy only
when the required responsibility or consumer contract actually changes.

Passing the numeric limit alone is not completion. The affected behavior and
consumer contracts must remain valid; the reported architectural violations
and applicable hard limits must be resolved through cohesive ownership.

A language's module entry filename is not an architectural responsibility.
For example, moving Rust `name.rs` to `name/mod.rs` can colocate an existing
owner with its implementation and tests, but does not by itself repair coupling.
Explain the ownership or navigation improvement independently of lint counts.
Inspect the entry's actual contents before describing it as a public facade;
keep cohesive implementation there when appropriate, and use responsibility
names when separating behavior. Do not require empty facades or ban standard
entry filenames. If a finding appears to reflect module wiring rather than
the intended architectural dependency, inspect the measured edge and analyzer
semantics before moving code solely to change its directory classification.

Inventory completeness and measured dependencies are mechanically checked.
Read the implementation to assess whether descriptions match its responsibility;
a green lint result does not prove semantic cohesion.

If no defensible partition preserves `C`, report the conflict instead of creating
arbitrary structure or weakening the contract.
