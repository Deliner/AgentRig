---
name: relocate-tests
description: Move a Rust test-only module file declared under #[cfg(test)] into a tests/ directory or a tests.rs file, so its tests see the same code and the rules measure it as test code.
---

# Keep test files where test code is expected

A test-only module file named after what it tests, such as `mesh_tests.rs`,
sits among production files. Every rule measures it as production code, so its
unwraps, long literals and fixtures become findings, and the project ends up
excluding its own file pattern from many rules. Test code belongs in a
`tests.rs` file or under a `tests/` directory, which the shipped rules exclude.

1. Read the declaration the finding names and what the tests rely on: their
   `super::` and `crate::` paths, the private items they reach, and every cargo
   test filter or affected-test target that names the module.
2. Move the file into a `tests/` directory beside the declaring file and keep
   the module name with `#[path]`, relative to the declaring file's directory:

   ```rust
   #[cfg(test)]
   #[path = "tests/mesh.rs"]
   mod mesh_tests;
   ```

   The module keeps its place in the tree, so `super::` paths, private access
   and test names such as `mesh_tests::` stay valid. Move with `git mv` and
   describe the new directory and file where the project inventories them,
   such as `architecture.yaml`.
3. When the file tests a single module, it may instead become that module's
   `tests.rs`: declare `#[cfg(test)] mod tests;` in the tested module and move
   the file to `mesh/tests.rs` for `mesh.rs`, or `world/tests.rs` for
   `world/mod.rs`. Its module path changes, so update `super::` paths in the
   tests and every filter or target that names the old module.
4. Remove the project's exclusions of the old file name pattern from rules; the
   shipped test exclusions now cover the file.
5. Build and run the moved tests, then rerun the reported lint check.

## Rule binding

The measurement flags each Rust `mod` declaration without a body that is gated
by `#[cfg(test)]`, including `all(test, ..)`, or declared inside a test-only
module, when no enclosing inline module is named `tests` and no component of
its `#[path]` value, or of its module name without one, is `tests` or
`tests.rs`. The finding is at
the declaration line and names the `#[path]` value, else the module name. Files
under the shipped exclude, `**/tests/**` and `**/tests.rs`, are not measured.
Other rules measure the declared file as before until it is moved. Do not
exclude the file or weaken the rule merely to pass. Apply complexity-discipline;
stop when every test-only file is in test position and its tests pass.
