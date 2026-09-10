---
name: complexity-discipline
description: >
  Use when designing or modifying non-trivial code, systems, or development
  workflows where the solution may silently strengthen instructions, expand
  scope, add distinctions, guarantees, mechanisms, lifecycle, validation, or
  process. Keeps the implementation semantically aligned with current
  requirements and limited to necessary complexity.
---

# Preserve intent with minimum necessary complexity

Achieve the required result under its existing constraints. Minimize unnecessary
complexity, not correctness, meaningful structure or required guarantees.

## Apply throughout the entire session

This is a continuous reasoning invariant, not a one-time check or gate. Apply it
to every reasoning step, proposed operation, implementation choice, scope change,
validation step and completion judgment from loading until the session ends.
Reassess whenever the requirement, evidence or approach changes.

Let R be the required result, C its preserved constraints and X a proposed
addition. At each decision:

1. Establish R and C from current authoritative sources.
2. Identify what X changes in behavior, obligations or structure.
3. Admit X only if removing it prevents R, violates C, or restores greater
   existing complexity that X directly consolidates without changing R or C.
4. Choose the lowest sufficient mechanism and reject unsupported expansion.
5. Verify the affected result and stop at the authorized completion boundary.

Apply this as a domain-independent relation between results, constraints,
evidence and changes. Domain terms and examples are supplied by the current task;
they do not define the rule. Any example is replaceable illustration.
Portability requires the same prescription after substituting another domain,
not wording compatible with every possible behavior.

## Ground every addition

Valid grounds are an explicit current requirement, authoritative contract,
established invariant, demonstrated or credible failure in the actual system,
concrete operational constraint, genuine security/trust/permission boundary,
or an objective fact needed to determine or perform the action.

Elegance, stronger assurance, known patterns, hypothetical reuse, possible future
needs, imaginable failures and the needs of another proposed mechanism are not
independent grounds. Approval for an action does not expand its scope.

Consolidation is valid when it replaces duplicated representations of the same
required behavior and reduces total complexity without changing semantics.
Do not silently turn R into a larger result or C into stronger constraints.

## Reject changes that only appear to improve the result

- **Preconditions:** do not make an already determined, authorized action depend
  on another proof or approval unless a valid source requires it. Desirable
  confidence is not a prerequisite.
- **Postconditions:** do not require another artifact, proof or state for
  completion unless it belongs to acceptance or necessary falsifiable
  verification.
- **Scope and guarantees:** do not add a complementary objective or strengthen
  a guarantee merely because it appears useful or safer.
- **Distinctions:** represent cases separately only when their required behavior,
  allowed operations, effects, transitions, lifecycle, security treatment or
  enforced invariant differs. Distinct names and examples do not imply distinct
  production states; tests may enumerate examples of one behavioral rule.
- **Variability:** add a mode or configuration dimension only when multiple
  behaviors are currently required.
- **Indirection:** an intermediate mechanism needs a current responsibility:
  owning an invariant or meaningful state, isolating a real boundary, unifying
  an existing duplicated decision or serving multiple current implementations.
- **Representation and lifecycle:** using information does not itself justify
  giving it identity, persistence, synchronization, recovery or a managed
  lifecycle. Each obligation needs support from the original requirement.
- **Failure handling:** address credible failures in the actual environment or
  contract. Logical possibility alone does not extend the failure model.
- **Responsibility:** keep enforcement at its authoritative owner. Revalidate at
  a genuine new trust boundary; fix an owner that can violate its guarantee
  rather than distributing defensive checks throughout trusted consumers.
- **Supporting machinery:** a mechanism's need for further machinery does not
  justify either. Trace each addition to R and C. First reconsider removing or
  simplifying the originating mechanism and its resulting obligations.
- **Verification and coordination:** do not turn support into a parallel product
  that recursively needs its own verification, planning or tracking. Use the
  cheapest falsifiable check adequate for the current task; reconsider support
  whose validation becomes substantial.
- **Time and exploration:** do not prepare a hypothetical future requirement.
  Investigate enough to understand owners, contracts, affected flow, consumers,
  trust boundaries and existing mechanisms; stop when that is sufficient.
  A small change based on misunderstanding is not minimal.
- **Completion:** achieving R under C does not authorize further rigor,
  documentation, resilience, generality, extensibility or preparation.

Required workflows, approval gates, security controls, compliance requirements
and verification contracts remain constraints. Simplification must preserve
them, together with required data protection, compatibility, accessibility,
concurrency and operational behavior. Less structure is not inherently better.

## Choose and finish

Prefer the first sufficient rung:

no change → existing behavior → existing operation or type → platform capability
→ installed dependency → small direct local implementation → new abstraction
→ new subsystem.

Before declaring completion, review each introduced behavior, condition, state,
distinction, mechanism, artifact, option, validation, guarantee, workflow, file
and dependency:

- Which valid source requires it, and what breaks if it is removed?
- Did its underlying problem exist independently, or did this design create it?
- Does it change the requested semantics, permissions or completion criteria?
- Can equivalent cases share one rule, the existing owner enforce it once, or
  a lower rung satisfy the same contract?

Remove or simplify unsupported additions. Stop when the requested behavior,
preserved constraints and adequate falsifiable verification are satisfied.
Among correct alternatives, choose the one with the fewest independently
meaningful behaviors, states, distinctions, mechanisms, obligations and degrees
of freedom.
