---
name: complexity-discipline
description: >
  Use when designing or modifying non-trivial code, systems, or development
  workflows where the solution may silently strengthen instructions, expand
  scope, add distinctions, guarantees, mechanisms, lifecycle, validation, or
  process. Keeps the implementation semantically aligned with current
  requirements and limited to necessary complexity.
---

# Complexity Discipline

Build the smallest system that correctly satisfies the current requirements.

The goal is not minimum code or maximum architectural purity. The goal is
**minimum necessary complexity without changing the semantics of the task**.

# IMPORTANT: APPLY THIS DISCIPLINE THROUGHOUT THE ENTIRE SESSION

This skill is a mandatory reasoning invariant for the entire active session.
Applying or refreshing it is not a one-time action, checklist, or gate. From
the moment it is loaded until the session ends, every reasoning step, proposed
operation, implementation choice, validation step, scope decision, and
completion judgment must conform to every applicable instruction below.

Use the skill as a domain-independent schema. It fixes the invariant structure
of the reasoning: the operations to perform, the relationships among elements,
the constraints to preserve, and the order in which decisions are evaluated.
It does not fix the domain-specific objects, terminology, or illustrative
content. Substitute those when applying the schema; do not change the schema to
fit them.

At every decision point:

1. Identify the current required result `R` and preserved constraints `C`.
2. Identify each proposed addition or change `X`.
3. Test `X` against the valid sources and prohibitions in this skill.
4. Reject unsupported semantic expansion and choose the lowest sufficient
   implementation rung.
5. Repeat the check whenever the approach or task state changes, and apply the
   final audit before declaring completion.

Evidence of portability: replace the subject domain with an arbitrary other
domain and the rule still works without revision; remove domain-specific nouns
and the logical operations, relationships, constraints, and ordering remain.

Boundary of portability: abstraction must not erase the prescription. The
schema must still require some reasoning and forbid some reasoning. A statement
compatible with every possible behavior is content-free, not portable.

Every example in this skill is replaceable illustration only. Examples help
instantiate the schema; they are not part of the invariant and do not limit its
scope.

## Preserve the original semantics

Let:

```text
R = the required result
C = the constraints that must be preserved
X = a proposed addition
```

Introduce `X` only when removing it would make `R` impossible, violate `C`, or
restore greater existing complexity that `X` directly consolidates without
changing `R` or `C`.

Replacing several existing representations of the same behavior with one rule
can reduce complexity even when the new rule is not itself a product
requirement. This is consolidation, not semantic expansion.

Valid sources for `X` are:

* an explicit current requirement;
* an existing authoritative contract;
* an established project invariant;
* a demonstrated or credible failure mode in the actual system;
* a concrete operational constraint;
* a genuine security, trust, or permission boundary;
* an objective fact without which the required action cannot be determined or
  performed.

The following are not independent justification:

* `X` seems cleaner, safer, stronger, or more rigorous;
* `X` may be useful in the future;
* a known pattern can represent `X`;
* another mechanism introduced by the same design needs `X`;
* extra confidence would be desirable;
* a scenario or failure can be imagined.

Do not silently transform `R` into a larger result or `C` into stronger
constraints. Approval to perform an action does not authorize expanding it.

## Invented preconditions

Do not strengthen an instruction by adding a condition that must be established
before the requested action may proceed.

Abstract form:

```text
Required:
perform A

Introduced:
verify P
perform A only if P is established
```

`P` is justified only when it follows from a valid source above. If `A` is
already determined and permitted, perform `A` directly.

Desirable confidence is not a prerequisite.

## Invented postconditions and completion criteria

Do not require an additional result, proof, or state before considering the
requested action complete.

Abstract form:

```text
Required:
perform A

Introduced:
perform A
produce or establish E
consider A complete only when E exists
```

`E` must be part of the required result, an authoritative acceptance contract,
or necessary falsifiable verification. Otherwise completion remains defined by
`A` itself.

## Scope inflation

Do not add a second objective to the requested result.

Abstract form:

```text
Required:
A

Introduced:
A + B
```

`B` is not justified merely because it complements, improves, documents,
organizes, or prepares `A`. Add it only if `A` cannot satisfy the original
requirement without it.

## Guarantee inflation

Do not silently replace a required property with a stronger one.

Abstract form:

```text
Required:
A with guarantee G

Introduced:
A with guarantee G'

where G' is stronger than G
```

The stronger guarantee requires its own current driver. General desirability,
defence in depth, hypothetical failures, or future compatibility are not enough.

## Distinction inflation

Do not give separate production representations to cases that have the same
required behavior.

Abstract form:

```text
Required behavior:
B(X) = B(Y)

Introduced:
represent X and Y separately
branch on the distinction
maintain both paths
```

Preserve a distinction only when it changes required observable behavior,
allowed operations, side effects, transitions, lifecycle, security treatment,
or an invariant that must be enforced.

A distinct name or example does not imply a distinct state. Tests may enumerate
examples even when production behavior should be compressed into one rule.

## Variability inflation

Do not turn one required behavior into a configurable family of behaviors.

Abstract form:

```text
Required:
A

Introduced:
A(mode)
```

A mode, flag, optional path, strategy, or configuration dimension is justified
only when multiple behaviors are currently required. Imagined alternatives do
not create present variability.

## Indirection inflation

Do not insert a mechanism between a requirement and its implementation unless
the mechanism has a current independent responsibility.

Abstract form:

```text
Required relation:
A -> B

Introduced:
A -> M -> B
```

`M` may be justified when it owns an invariant, controls meaningful state,
isolates a genuine boundary, centralizes an existing duplicated decision, or
supports multiple current implementations. Symmetry, possible replacement,
generic reuse, and pattern conformity are not independent responsibilities.

## Representation and lifecycle inflation

Do not turn transient information into an independently managed entity without
a current need for that identity or lifetime.

Abstract form:

```text
Required:
produce or use X

Introduced:
identify X
store X
track X
manage changes to X
recover X
```

Using `X` now does not imply that `X` must persist. Once identity or persistence
is introduced, its synchronization, invalidation, cleanup, compatibility, and
recovery obligations are additional complexity and require independent support
from the original requirement.

## Failure-model inflation

Handle failures credible in the actual system. Do not expand the design around
merely imaginable failures.

Abstract form:

```text
Established failure model:
F

Introduced failure model:
F union H

where H is hypothetical
```

`H` requires evidence from the actual environment, threat model, trust boundary,
external contract, or observed system behavior. Logical possibility alone is
not evidence.

## Responsibility diffusion

Do not distribute enforcement of an invariant already owned by an authoritative
component.

Abstract form:

```text
Established:
owner O guarantees P

Introduced:
consumer C1 checks P
consumer C2 checks P
consumer C3 checks P
```

Validate again only at a genuine new trust boundary. If `P` can actually be
violated inside the trusted system, fix its authoritative owner instead of
adding defensive behavior throughout its consumers.

## Supporting-machinery cascade

Supporting machinery does not inherit justification from the mechanism it
supports.

Abstract form:

```text
Required:
A

Introduced:
M1 for A
M2 for M1
M3 for M2
```

The need for `M2` to operate `M1` does not prove that either mechanism is needed
for `A`. Trace every mechanism back to the original requirement. If a mechanism
creates substantial supporting obligations, reconsider that mechanism before
satisfying those obligations.

Prefer removing originating complexity over completing the ecosystem around it.

## Meta-recursion

Do not let verification, coordination, or another supporting activity become a
parallel product that recursively requires the same kind of support.

Abstract form:

```text
Required:
establish A

Introduced:
establish V(A)
establish V(V(A))
maintain the mechanisms around V
```

Use the cheapest falsifiable check that gives adequate confidence for the
current change. If validating the validation becomes a substantial problem,
reconsider the original validation mechanism.

Apply the same rule to planning, delegation, review, evidence, coordination, and
progress tracking. Development machinery must justify itself by reducing total
reasoning or execution cost for the current task.

Required project workflows, approval gates, security controls, compliance
requirements, and verification contracts remain authoritative constraints.

## Temporal scope inflation

Do not expand a current requirement with preparation for a hypothetical future
requirement.

Abstract form:

```text
Required now:
A

Introduced:
A + preparation for possible B
```

The possibility of `B` does not make it current. Design `B` when it becomes a
real requirement and its actual constraints are known.

## Exploration inflation

Investigate enough to identify the authoritative owner, affected flow, relevant
contracts, downstream behavior, trust boundaries, and existing mechanisms.

Abstract form:

```text
Needed knowledge:
K

Investigation expands to:
K + unrelated U
```

Stop when `K` is sufficient to make the change correctly. Do not turn nearby
code, possible cleanup, or architectural curiosity into additional scope.

A small change based on misunderstanding is not minimal. Unbounded exploration
is not minimal either.

## Implementation-rung inflation

Stop at the first implementation level that fully satisfies the requirement.

Abstract form:

```text
Lower mechanism L satisfies R and C

Introduced instead:
higher mechanism H
```

Prefer, in order:

```text
no change
-> existing behavior
-> existing operation or type
-> platform capability
-> installed dependency
-> small direct local implementation
-> new abstraction
-> new subsystem
```

Do not select `H` because it appears more general, elegant, reusable, complete,
or future-ready.

## Compensatory complexity

When a newly introduced mechanism becomes difficult to maintain, reconsider the
mechanism before adding machinery to manage its consequences.

Abstract form:

```text
Introduced:
M

M creates obligation O

Proposed response:
add N to manage O
```

First test whether removing or simplifying `M` eliminates `O`. Do not treat a
problem created by the design as an immutable product requirement.

## Completion drift

Stop when the requested behavior is implemented, authoritative constraints are
preserved, and adequate falsifiable verification passes.

Abstract form:

```text
Reached:
R satisfies C

Continued work:
strengthen, generalize, formalize, document, or prepare R further
```

Completion does not authorize additional rigor, architecture, resilience,
evidence, extensibility, or future readiness.

## Do not erase necessary constraints

Complexity discipline does not mean removing structure indiscriminately.

Abstract form:

```text
Required:
R subject to C

Incorrect simplification:
R without C
```

Preserve every constraint supported by a valid source, including required
security, trust-boundary validation, data protection, compatibility,
accessibility, concurrency, and operational behavior.

The objective is not less architecture. The objective is no semantic expansion
or complexity without a current job.

## Final audit

For every newly introduced behavior, condition, state, distinction, mechanism,
artifact, option, validation, guarantee, workflow, file, or dependency, ask:

1. What valid source requires it?
2. Would removing it prevent the required result, violate an authoritative
   constraint, or restore greater existing complexity that it consolidates?
3. Did the problem exist before this design, or was it created by the design?
4. Is its justification independent, or inherited from another new mechanism?
5. Did it strengthen the instruction, completion criteria, or guarantees?
6. Can multiple cases be collapsed into one behavioral rule?
7. Can an existing owner enforce the property once?
8. Does a lower implementation rung already satisfy the requirement?

Remove or simplify anything without an independent current justification.

When several implementations are correct, choose the one that preserves `R` and
`C` while introducing the fewest independently meaningful behaviors, states,
distinctions, mechanisms, obligations, and degrees of freedom.
