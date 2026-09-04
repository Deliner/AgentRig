---
name: reduce-parameters
description: Simplify function, method or constructor inputs reported by parameter-count without hiding dependencies.
---

Read the declaration and call sites; fix syntax first if parsing failed. The rule counts declared inputs, including optional and variadic parameters, while excluding method receivers. Constructors are checked through Python __init__/__new__ and Rust associated functions; call-site argument counts and generated constructors are not inferred.

Remove redundant inputs or group values only when they already describe one meaningful concept. Keep explicit dependencies and update callers, defaults and external compatibility together. Do not introduce an untyped bag, variadic signature or new class solely to hide the count.

Preserve evaluation order and behavior. Rerun just lint and relevant caller tests; apply complexity-discipline before adding an abstraction.
