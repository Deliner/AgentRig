// DECISION: D157
use crate::shared::testing::{Run, configure, git, invoke, python_path, run, runtime};
use std::{fs, path::Path, path::PathBuf};
use tempfile::TempDir;

const MANIFEST: &str = "[package]\nname = \"sample\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";
const COMMAND: &str = "test:
  purpose: Run the project library tests
  argv: [cargo, test, --manifest-path, Project/Cargo.toml, --target-dir, target, --lib, --]
  accepts_args: true
";
const DETAIL: &str = "# I001\n\n## Predicate\n\nDoubles correctly.\n\n## Oracle\n\nRun I001.\n";

struct Runner {
    language: &'static str,
    runner: &'static str,
    target: &'static str,
    file: &'static str,
    function: &'static str,
    marker: &'static str,
    valid: &'static str,
    misbound: &'static [&'static str],
    missing: &'static str,
    undiscoverable: &'static str,
    undiscovered: &'static str,
    discovery: &'static str,
}

const PYTEST: Runner = Runner {
    language: "python",
    runner: "pytest",
    target: "Project/test_sample.py::test_doubles",
    file: "Project/test_sample.py",
    function: "test_doubles",
    marker: "# INVARIANT: I001",
    valid: "# INVARIANT: I001\ndef test_doubles():\n    pass\n",
    misbound: &[
        "# INVARIANT: I001\ndef unrelated():\n    pass\n",
        "class Other:\n    # INVARIANT: I001\n    def test_doubles(self):\n        pass\n",
    ],
    missing: "marked oracle function",
    undiscoverable: "__test__ = False\n# INVARIANT: I001\ndef test_doubles():\n    pass\n",
    undiscovered: "test discovery failed",
    discovery: "--collect-only",
};

const CARGO: Runner = Runner {
    language: "rust",
    runner: "cargo",
    target: "feature::tests::doubles",
    file: "Project/src/feature/tests.rs",
    function: "doubles",
    marker: "// INVARIANT: I001",
    valid: "// INVARIANT: I001\n#[test]\nfn doubles() {}\n",
    misbound: &["// INVARIANT: I001\nfn unrelated() {}\n"],
    missing: "was not discovered",
    undiscoverable: "// INVARIANT: I001\nfn doubles() {}\n",
    undiscovered: "was not discovered",
    discovery: "--list",
};

fn project(runner: &Runner, tests: &str) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    git(&root, &["init", "-q", "-b", "trunk"]);
    let initialized = invoke(&root, &["init", "--language", runner.language]);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    let crate_root = runner.runner == "cargo";
    if crate_root {
        configure(&root, "commands", COMMAND);
        fs::create_dir_all(root.join("Project/src/feature")).unwrap();
        fs::write(root.join("Project/Cargo.toml"), MANIFEST).unwrap();
        fs::write(root.join("Project/src/lib.rs"), "mod feature;\n").unwrap();
        let module = root.join("Project/src/feature/mod.rs");
        fs::write(module, "#[cfg(test)]\nmod tests;\n").unwrap();
    }
    bind(&root, runner, runner.target);
    write(&root, runner, tests);
    let index = root.join("Ledger/Invariants.md");
    let row = format!(
        "| [I001](Invariants/001.md) | Doubles correctly | [{}](../{}) |\n",
        runner.function, runner.file
    );
    fs::write(&index, fs::read_to_string(&index).unwrap() + &row).unwrap();
    fs::create_dir_all(root.join("Ledger/Invariants")).unwrap();
    fs::write(root.join("Ledger/Invariants/001.md"), DETAIL).unwrap();
    (temp, root)
}

fn bind(root: &Path, runner: &Runner, target: &str) {
    let oracle = format!(
        "I001: {{check: tests, runner: {}, target: '{target}'}}\n",
        runner.runner
    );
    configure(root, "oracles", &oracle);
}

fn write(root: &Path, runner: &Runner, tests: &str) {
    let path = root.join(runner.file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, tests).unwrap();
}

fn memory_check(root: &Path) -> Run {
    run(
        runtime(root, &["memory-check"]).env("PATH", python_path()),
        "",
    )
}

fn rejected(root: &Path, expected: &str) {
    let checked = memory_check(root);
    assert_eq!(checked.code, Some(2), "{expected}: {}", checked.output());
    assert!(
        checked.stderr.contains(expected),
        "{expected}: {}",
        checked.stderr
    );
}

// INVARIANT: I002
#[test]
fn invariant_oracle_must_name_a_discovered_test_carrying_its_marker() {
    for runner in [&PYTEST, &CARGO] {
        let (_temp, root) = project(runner, runner.valid);
        let checked = memory_check(&root);
        assert_eq!(checked.code, Some(0), "{}", checked.output());
        let unmarked = runner.valid.replace(runner.marker, "");
        for misbound in runner.misbound {
            write(&root, runner, &format!("{misbound}{unmarked}"));
            rejected(&root, "marked oracle function");
        }
        write(&root, runner, runner.valid);
        bind(
            &root,
            runner,
            &runner.target.replacen("::", "::Missing::", 1),
        );
        rejected(&root, runner.missing);
        bind(&root, runner, runner.target);
        write(&root, runner, runner.undiscoverable);
        rejected(&root, runner.undiscovered);
        let jobs = invoke(&root, &["jobs"]);
        assert!(jobs.stdout.contains(runner.discovery), "{}", jobs.output());
    }
}

#[test]
fn pytest_oracle_rejects_a_repeated_name_and_accepts_parametrized_markers() {
    let repeated = format!("{}\ndef test_doubles():\n    pass\n", PYTEST.valid);
    let (_temp, root) = project(&PYTEST, &repeated);
    rejected(&root, "marked oracle function");
    let decorator = "@pytest.mark.parametrize(\"unused\", [1, 2])";
    let parametrized = PYTEST
        .valid
        .replace("test_doubles()", "test_doubles(unused)");
    for marked in [
        format!("{}\n{decorator}", PYTEST.marker),
        format!("{decorator}\n{}", PYTEST.marker),
    ] {
        let tests = parametrized.replace(PYTEST.marker, &marked);
        write(&root, &PYTEST, &format!("import pytest\n{tests}"));
        let checked = memory_check(&root);
        assert_eq!(checked.code, Some(0), "{tests}: {}", checked.output());
    }
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
        let (_temp, root) = project(&CARGO, &tests);
        rejected(&root, "marked oracle function");
    }
}
