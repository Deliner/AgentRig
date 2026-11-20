---
name: relocate-tests
description: Move Rust or Python test code outside a tests/ directory, a test-only module file, a tests.rs file or a test_*.py or *_test.py file, into the tests/ directory of the code it tests, so its tests see the same code and the rules measure it as test code.
---

# Keep tests in the tests directory

Test code outside a `tests/` directory sits among production files. Every rule
measures it as production code, so its unwraps, long literals and fixtures
become findings, and the project ends up excluding its own file patterns from
many rules. The tests of a directory live in its `tests/` directory, which the
shipped rules exclude, so they are always in one place and the set grows there
without moving again.

1. Read the file or declaration the finding names and what the tests rely on:
   their `super::`, `crate::` or import paths, the private items they reach,
   and every test filter, runner setting or affected-test target that names
   them.
2. Move a Rust `tests.rs` to `tests/mod.rs` beside it with `git mv`. A
   declaration `#[cfg(test)] mod tests;` in the directory entry, `mod.rs`,
   `lib.rs` or `main.rs`, stays as it is, so the module path, the test names
   and `super::` paths stay valid.
3. Move a Rust test-only module file with another name, such as
   `mesh_tests.rs`, into the `tests/` directory of its directory as a file
   named after the behavior it verifies, such as `tests/mesh.rs`, and declare
   it from `tests/mod.rs`, which the entry declares as
   `#[cfg(test)] mod tests;`. Test code may be declared only there:
   `test-in-code-file` reports a test declaration in any other file. When the
   tests reach private items of the file that declared them, apply
   move-test-code-out.

4. Move a Python `test_*.py` or `*_test.py` file into the `tests/` directory of
   the package it tests, and update its imports and every runner setting that
   names the old path.
5. When the tests of one directory grow large, split them by the behavior they
   verify into files inside `tests/`, declared from `tests/mod.rs` in Rust.
6. Give each new `tests/` directory the inventory the project keeps, such as an
   `architecture.yaml` with its purpose and files, and list it among the
   directories of its parent.
7. Remove the project's exclusions of the old file patterns from rules; the
   shipped test exclusion `**/tests/**` covers the moved files. Build and run
   the moved tests, then rerun the reported lint check.

## Rule binding

The measurement flags each Rust file named `tests.rs` and each Python file
named `test_*.py` or `*_test.py` when no component of its path is `tests`, at
line 1 naming the file. It also flags each Rust `mod` declaration without a
body that is gated by `#[cfg(test)]`, including `all(test, ..)`, or declared
inside a test-only module, when no enclosing inline module is named `tests` and
no component of its `#[path]` value, or of its module name without one, is
`tests`; that finding is at the declaration line and names the `#[path]` value,
else the module name. Files under the shipped exclude `**/tests/**` are not
measured. Do not exclude the file or weaken the rule merely to pass. Apply
complexity-discipline; stop when every test file is in a `tests/` directory and
its tests pass.
