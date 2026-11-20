---
name: move-test-code-out
description: Move Rust test code out of a code file into the tests/ directory of its directory, declared in the directory entry, so tests call only what the file opens to its directory and never its private items.
---

# Test a file through what it opens

Test code inside a code file sees every private item of that file. Its tests
then pin the file's internals: a helper renamed, a field split or a function
inlined breaks tests although no caller sees a change, and helpers built from
private fields hide construction paths that real callers cannot take. Tests of
a directory live in its `tests/` directory and see only what each file opens
to the directory, which is what the rest of the directory uses too.

1. Read the reported item and everything it uses: the private functions,
   fields and constructors it reaches, its `super::` and `crate::` paths, and
   every test filter, runner setting or affected-test target that names its
   tests.
2. Move the tests into the `tests/` directory of the file's directory. Put
   them in a file named after the behavior they verify, declare it from
   `tests/mod.rs`, and declare `tests/` in the directory entry, `mod.rs`,
   `lib.rs` or `main.rs`, as `#[cfg(test)] mod tests;`. Give a new `tests/`
   directory its `architecture.yaml` and list it in the parent's.
3. Call only what the file opens to its directory: `pub(super)` or wider
   items of the file, and items of the entry itself.
4. Test a private function through the open function that uses it, with
   inputs that reach each case the old test covered.
5. Replace a test helper that builds a value from private fields with
   construction through open functions, the same way callers build it.
6. Never open a function, field or constructor only for a test. When a case
   cannot be reached through the open surface, it is either unreachable for
   every caller and its code can go, or the file hides a behavior its callers
   need, which is a design change to make on its own merits.
7. Update the moved test paths in filters, runner settings and test maps,
   delete the old test code, build and run the moved tests, then rerun the
   reported lint check.

## Rule binding

The measurement flags every outermost Rust item gated by `#[cfg(test)]`,
including `all(test, ..)`, or marked by an attribute whose last path segment
is `test`, such as `#[test]` or `#[tokio::test]`, in a file outside a `tests`
directory, at the item's line and naming it. Items nested in a flagged item
are not reported again. The one item it accepts is a module declaration
without a body in a directory entry `mod.rs`, `lib.rs` or `main.rs`. Inner
attributes such as `#![cfg(test)]` and the `cfg!(test)` macro are not
measured. Files under the shipped exclude `**/tests/**` are not measured. Do
not exclude the file or weaken the rule merely to pass. Apply
complexity-discipline; stop when the code file has no test code and the moved
tests pass.
