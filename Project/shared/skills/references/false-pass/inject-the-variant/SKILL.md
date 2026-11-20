---
name: inject-the-variant
description: Replace a Rust item compiled only outside tests, or its cfg(test) twin, with a value the caller and the test both supply, or record why tests must not compile the item.
---

# Inject what varies instead of compiling it out of tests

An item under `#[cfg(not(test))]`, or under any `cfg` predicate that is false
whenever `test` is set, is never compiled into a test build. Tests therefore
never run it, and when a `cfg(test)` twin with the same name takes its place,
the tests check different code than the product runs. A defect in the real item
stays invisible to every test.

1. Read the item and its twin, if any, and name what differs between the
   product and the test: a clock, a directory, a thread pool, a file system, a
   network client or an environment value.
2. Pass that difference in from outside: a function parameter, a constructor
   argument, a field, or a trait value that the product caller and the test
   both supply. Keep one implementation of the item and compile it in every
   build.
3. Delete the `cfg(test)` twin once the test supplies its own value, and make
   the test exercise the same item the product calls.
4. A `use`, `extern crate` or `type` without a `cfg(test)` twin runs no code
   of its own: remove the `cfg` when the name is used in every build. When the
   item genuinely cannot exist in a test build, such as code the test harness
   provides itself, record the reasoning as a decision with edit-decisions and
   put its exception `// DECISION: DNNN RULE`, RULE being the rule ID the
   finding names, alone on the line directly above the `cfg` attribute. List it
   in the decision as `[exception RULE](FILE)`; the user approves a new
   exception through propose-protected-change.
5. Rerun the reported lint check and the affected behavior checks.

An exception naming this rule, `test-substitution` by default, on the line
directly above the attribute keeps it; a bare `DECISION:` or `INVARIANT:`
reference lifts nothing.

## Rule binding

The measurement flags every Rust `#[cfg(..)]` or `#![cfg(..)]` attribute of an
item in a file, module, impl or trait body whose predicate is false in every
build with `test` set, such as `not(test)`, `all(not(test), ..)` or
`not(any(test, ..))`, outside test code. `cfg_attr`, `any(not(test), ..)`,
statements and fields are out of scope. Rerun the reported lint check and
relevant behavior checks. Do not weaken the rule or its selectors merely to
pass. Apply complexity-discipline; stop when the item compiles in test builds
with its variation injected, or an approved exception lifts the finding.
