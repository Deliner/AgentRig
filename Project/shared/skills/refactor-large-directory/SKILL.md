---
name: refactor-large-directory
description: Choose responsibility and change boundaries before structural work, or repair directory size, architecture inventory and dependency findings while preserving behavior.
---

# Organize by responsibility and reason to change

Make a small part of the structure sufficient to identify its responsibility,
what can change independently, how consumers reach it and the minimum context
needed to continue work. Reduce the cost of finding, reading and reasoning about
the relevant behavior while preserving its contract.

Read the task or finding, affected architecture.yaml contracts and enclosing
boundaries. Establish behavior, consumers, state, tests, actual dependencies and
required path compatibility before choosing a partition. Technology, existing
entity names or historical location alone do not establish ownership.

## Semantic roles

These roles describe responsibility and dependency relationships. They do not
prescribe directory names, a fixed tree or a separate container for each role.

- Area: a broad responsibility whose changes share a common cause.
- Unit: the smallest autonomous behavior that can be understood, changed and
  verified locally; autonomy does not require copying the capabilities it uses.
- Public: the unit's sole surface for external interaction.
- Application: scenario orchestration and coordination of internal capabilities.
- Core: rules and models meaningful independently of invocation and infrastructure.
- Adapters: the connection between the internal model and the external environment.
- Tests: local verification of the unit's contract and behavior.
- Capability: a responsibility with independent meaning and multiple consumers.

Choose physical structure according to actual behavioral complexity. A simple
unit may keep its implementation together; separate internal roles only when
they need independently meaningful boundaries. Containers may have different
depths while components with the same role retain equivalent architectural
relationships. Every additional level must have a responsibility explainable
without referring to its position in the tree.

## Ownership and dependencies

Keep behavior, data, adapters and tests with their owner and common reason to
change. Owning a scenario does not imply owning every capability it uses. Shared
behavior needs an independent responsibility before a shared location; reuse
alone does not justify shared/common/utils buckets.

Direct dependencies from concrete details toward stable responsibilities:
external toward internal, consumer toward capability, implementation toward
contract. Consumers use the owner's public surface without reaching through to
internal parts. Introduce a contract or abstraction only for a real relationship;
the role vocabulary does not require an extra interface or build package.

Keep the actual architectural dependency graph acyclic. Direct and indirect
cycles are violations. Check measured source dependencies; the graph of allowed
dependencies does not establish whether a cycle exists in the code.

The structure, its description and actual dependencies must express the same
ownership. A wider description cannot justify an accidental arrangement.

## Choose the relevant repair

- Missing or stale description: establish intended ownership from implementation
  and consumers before updating it. Do not approve misplaced elements merely
  by recording their current location.
- Excessive size: use the ownership and semantic roles above to find the
  smallest meaningful partition that resolves the finding.
- Forbidden dependency, private access or cycle: trace actual consumers and
  targets. Repair the relationship at its owner, use an existing public boundary,
  or relocate misplaced responsibility or a genuinely shared contract when that
  corrects dependency direction. Add an interface only for a current need.
- Incomplete analysis: inspect the reported source and resolver limits. An
  unresolved local relationship cannot be called external or excluded merely
  to obtain a clean graph.

Move complete responsibilities when needed and update directly affected
consumers and references. Do not split by position or count, rename unrelated
parts, or add layers for symmetry. A separate build package requires a real
dependency, infrastructure, build or test boundary. Do not widen allow/public,
remove deny constraints, hide cycles behind reexports or relax exclusions merely
to pass. Policy changes require an actual change in responsibility or contract.

## Repository binding and completion

Give each scoped directory one line of purpose. Register files individually with
what belongs in each and describe children in the separate directories block;
each child has its own architecture.yaml contract. Express ownership and permitted
relationships through these contracts without requiring role-named directories.

Standard module entry filenames do not establish ownership. Inspect their actual
contents before calling them facades. Keep cohesive implementation there when
appropriate; do not require empty entries or ban standard names. When a finding
may describe module wiring rather than an architectural dependency, inspect the
measured edge and analyzer semantics before moving code.

Enforce declared ownership boundaries, dependency direction, public access and
cycle constraints through the supported checks. Depth, extensive sharing,
abstraction and asymmetry prompt analysis; they are not violations by themselves.
Preserve configured hard size limits. Inventory and measured edges are mechanical
evidence; the meaning of ownership requires inspecting implementation and consumers.

Rerun lint and affected behavior checks. Verify that responsibility, independent
change and the public boundary can be recovered from a small relevant part of
the structure without tracing arbitrary fragments. Stop when the authorized
structural outcome or reported violation is resolved with preserved contracts.
If no defensible repair preserves them, report the conflict. Apply
complexity-discipline; further possible decomposition is not further required work.
