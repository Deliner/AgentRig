// DECISION: D157
use crate::shared::testing::{configure, git, invoke};
use std::{fs, path::PathBuf};
use tempfile::TempDir;

const MANIFEST: &str = "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";
const COMMAND: &str = "test:
  purpose: Run the project library tests
  argv: [cargo, test, --manifest-path, Project/Cargo.toml, --target-dir, target, --lib, --]
  accepts_args: true
";
const ORACLE: &str = "I001: {check: tests, runner: cargo, target: feature::tests::doubles}\n";
const ROW: &str = "| [I001](Invariants/001.md) | Doubles correctly | [doubles](../Project/src/feature/tests.rs) |\n";
const DETAIL: &str = "# I001\n\n## Predicate\n\nDoubles correctly.\n\n## Oracle\n\nRun I001.\n";

fn project(tests: &str) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    git(&root, &["init", "-q", "-b", "trunk"]);
    let initialized = invoke(&root, &["init", "--language", "rust"]);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    configure(&root, "commands", COMMAND);
    configure(&root, "oracles", ORACLE);
    let feature = root.join("Project/src/feature");
    fs::create_dir_all(&feature).unwrap();
    fs::write(root.join("Project/Cargo.toml"), MANIFEST).unwrap();
    fs::write(root.join("Project/src/lib.rs"), "mod feature;\n").unwrap();
    fs::write(feature.join("mod.rs"), "#[cfg(test)]\nmod tests;\n").unwrap();
    fs::write(feature.join("tests.rs"), tests).unwrap();
    let index = root.join("Ledger/Invariants.md");
    fs::write(&index, fs::read_to_string(&index).unwrap() + ROW).unwrap();
    fs::create_dir_all(root.join("Ledger/Invariants")).unwrap();
    fs::write(root.join("Ledger/Invariants/001.md"), DETAIL).unwrap();
    (temp, root)
}

#[test]
fn cargo_oracle_binds_the_marked_test_of_a_nested_module() {
    let (_temp, root) = project("// INVARIANT: I001\n#[test]\nfn doubles() {}\n");
    let checked = invoke(&root, &["memory-check"]);
    assert_eq!(checked.code, Some(0), "{}", checked.output());
}

#[test]
fn cargo_oracle_rejects_another_scope_a_partial_segment_and_an_ambiguous_name() {
    let unmarked = "#[test]\nfn doubles() {}\n";
    let other = "mod other {\n    // INVARIANT: I001\n    fn doubles() {}\n}\n";
    let partial = "// INVARIANT: I001\nfn oubles() {}\n";
    let ambiguous = "// INVARIANT: I001\nfn doubles() {}\nmod tests {\n    fn doubles() {}\n}\n";
    for tests in [
        other.to_owned() + unmarked,
        partial.to_owned(),
        ambiguous.to_owned(),
    ] {
        let (_temp, root) = project(&tests);
        let checked = invoke(&root, &["memory-check"]);
        assert_eq!(checked.code, Some(2), "{tests}: {}", checked.output());
        assert!(
            checked.stderr.contains("marked oracle function"),
            "{tests}: {}",
            checked.stderr
        );
    }
}
