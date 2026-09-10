---
name: edit-invariants
description: Maintain observable predicates linked to exact executable tests.
---

# Bind an enduring requirement to evidence that can refute it

An invariant states an observable property that must hold within a defined scope.
Its check must distinguish that property from a superficially successful result.
Apply norm-or-choice.

Establish the predicate and the exact verification that would fail if it were
violated. Keep the requirement, marked test and executable selection aligned.
Finding a test is not evidence that it ran; a passing unrelated or weaker test
does not establish the predicate.

Preserve identity. Change a predicate through an explicit decision and update
its verification consistently; do not weaken either side merely to obtain a
passing result. If the evidence is unavailable, retain that distinction rather
than reporting verification.

## Repository binding

Keep stable INNN IDs and Predicate/Oracle details. Link the test function and
place INVARIANT: INNN before its declaration. Configure the exact runner target
and check in agentrig.yaml, including class/module scope. Verify discovery and
execution of that target.
