---
name: refactor-large-directory
description: Repair directory size, architecture inventory and dependency findings through feature ownership, cohesive modules and public boundaries, preserving consumers and behavior.
---

# Restore correspondence between responsibility and structure

The structure, its description and actual dependencies must express the same
ownership. Resolving a finding must not merely make the inventory describe an
accidental arrangement.

Read the finding, affected architecture.yaml contracts and enclosing boundaries.
Establish the observable contract, consumers, path constraints, configuration,
tests and actual dependencies. Treat declared access and dependency boundaries
as intent, not obstacles to remove.

## Choose the relevant repair

- Missing or stale description: establish intended ownership from implementation
  and consumers before updating it. Do not approve misplaced elements merely
  by recording their current location.
- Excessive size: group elements by the behavior, invariant, lifecycle or reason
  to change they share. Separate only independently meaningful responsibilities;
  use the smallest partition that resolves the finding.
- Forbidden dependency, private access or cycle: trace actual consumers and
  targets. Repair the relationship at its owner, use an existing public boundary,
  or relocate misplaced responsibility or a genuinely shared contract when that
  corrects dependency direction. Add an interface only for a current need.
- Incomplete analysis: inspect the reported source and resolver limits. An
  unresolved local relationship cannot be called external or excluded merely
  to obtain a clean graph.

## Preserve meaning across the repair

Keep implementation, data, adapters and behavior tests with the feature or module
that owns them. Ownership of a scenario does not imply ownership of every
capability it uses. Shared behavior retains an independent shared owner;
consumers reach it through its public boundary.

Create a subcontainer only for a meaningful responsibility. Move complete groups
when needed, update directly affected references, and preserve old paths only
when consumers require compatibility. Do not prescribe depth, unit counts or
build packages for navigation alone. A separate build package requires a real
dependency, infrastructure, build or test boundary.

Do not group by name, position, representation or count alone; create generic
buckets, single-element layers or reciprocal dependencies; perform unrelated
renames; or add layers solely for symmetry. Do not widen allow/public patterns,
remove deny constraints, hide cycles behind reexports or relax exclusions just
to pass. Change policy only when the required responsibility or consumer
contract actually changes.

## Repository binding and completion

Give each scoped directory one line of purpose. Register files individually with
what belongs in each and describe children in the separate directories block;
each child has its own architecture.yaml contract.

Standard module entry filenames do not establish ownership. Inspect their actual
contents before calling them facades. Keep cohesive implementation there when
appropriate; do not require empty entries or ban standard names. When a finding
may describe module wiring rather than an architectural dependency, inspect the
measured edge and analyzer semantics before moving code.

Rerun lint and affected behavior checks; verify consumers, public access and
dependency direction. Inventory completeness and measured edges are mechanical
checks; semantic cohesion requires reading the implementation. Stop when the
reported violations and hard limits are resolved through cohesive ownership
with preserved contracts. If no defensible repair preserves them, report the
conflict instead of arbitrary restructuring. Apply complexity-discipline.
