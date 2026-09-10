---
name: reduce-parameters
description: Simplify function, method or constructor inputs reported by parameter-count without hiding dependencies.
---

# Preserve explicit meaning while reducing inputs

Reduce independently supplied information without hiding dependencies or
changing what callers can express.

1. Read the declaration and consumers. Establish each input's meaning,
   dependency and role in the observable contract.
2. Remove information already derivable without changing behavior. Combine
   values only when they already belong to one concept with a shared purpose;
   co-occurrence in a call does not establish that relationship.
3. Preserve explicit dependencies, evaluation order, defaults and external
   compatibility. Update the affected callers together.

A smaller signature is insufficient if unrelated inputs were hidden inside a
container, variadic channel or new type. Apply complexity-discipline before
adding an abstraction. Stop when the limit is met without concealing obligations
and relevant caller checks confirm preserved behavior.

## Rule binding

Fix syntax first if parsing failed. The rule counts declared inputs, including
optional and variadic parameters, excluding method receivers. Constructors are
checked through Python __init__/__new__ and Rust associated functions; call-site
counts and generated constructors are not inferred. Rerun the reported lint
check and relevant caller tests.
