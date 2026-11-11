// DECISION: D124
// DECISION: D138
use super::write;
use crate::shared::testing::{REPAIR, invoke, project};

type Group = (&'static [&'static str], &'static [&'static str]);

struct Case {
    argv: &'static [&'static str],
    groups: &'static [Group],
    findings: &'static [(&'static str, &'static str)],
    cwd: &'static str,
    include: &'static [&'static str],
}

const CARGO: &[&str] = &["cargo", "test", "--lib", "--"];
const PYTEST: &[&str] = &["uv", "run", "pytest"];
const SILENT_RUST: &str = "map it to '::' (every test)";
const SILENT_PYTHON: &str = "map it to '.' (every test)";
const UNSELECTED: &str = "the gate never selects it";
const RUST: Case = Case {
    argv: CARGO,
    groups: &[],
    findings: &[],
    cwd: ".",
    include: &["**"],
};
const PYTHON: Case = Case {
    argv: PYTEST,
    ..RUST
};
const CASES: [(&str, Case); 14] = [
    (
        "rust-covered",
        Case {
            groups: &[(&["src/**"], &["core::"])],
            ..RUST
        },
    ),
    (
        "rust-named-test",
        Case {
            groups: &[(&["src/**"], &["core::mesh_tests::mesh"])],
            ..RUST
        },
    ),
    (
        "rust-unmatched-runs-everything",
        Case {
            groups: &[(&["src/core/**"], &["core::"])],
            ..RUST
        },
    ),
    (
        "rust-silent-source",
        Case {
            groups: &[(&["**"], &[]), (&["src/core/**"], &["core::"])],
            findings: &[("src/lib.rs", SILENT_RUST)],
            ..RUST
        },
    ),
    (
        "rust-unselected-test",
        Case {
            groups: &[(&["src/**"], &["core::other::"])],
            findings: &[("src/core/mesh_tests.rs", UNSELECTED)],
            ..RUST
        },
    ),
    (
        "rust-test-outside-check-include",
        Case {
            groups: &[(&["src/**"], &["core::other::"])],
            include: &["src/lib.rs"],
            ..RUST
        },
    ),
    (
        "rust-unselected-test-inside-check-include",
        Case {
            groups: &[(&["src/**"], &["core::other::"])],
            findings: &[("src/core/mesh_tests.rs", UNSELECTED)],
            include: &["src/core/**"],
            ..RUST
        },
    ),
    (
        "python-covered",
        Case {
            groups: &[(&["**"], &["tests"])],
            ..PYTHON
        },
    ),
    (
        "python-nodes",
        Case {
            groups: &[(
                &["**"],
                &["tests/unit/test_a.py::test_a", "tests/test_b.py"],
            )],
            ..PYTHON
        },
    ),
    (
        "python-command-directory",
        Case {
            groups: &[(&["**"], &["unit", "test_b.py"])],
            cwd: "tests",
            ..PYTHON
        },
    ),
    (
        "python-silent-source",
        Case {
            groups: &[(&["src/**"], &[]), (&["tests/**"], &["tests"])],
            findings: &[("src/app.py", SILENT_PYTHON)],
            ..PYTHON
        },
    ),
    (
        "python-silent-source-outside-check-include",
        Case {
            groups: &[(&["src/**"], &[]), (&["tests/**"], &["tests"])],
            include: &["tests/**"],
            ..PYTHON
        },
    ),
    (
        "python-sibling-unselected",
        Case {
            groups: &[(&["**"], &["tests/test_b.py"])],
            findings: &[("tests/unit/test_a.py", UNSELECTED)],
            ..PYTHON
        },
    ),
    (
        "unknown-runner-unchecked",
        Case {
            argv: &["make", "check"],
            groups: &[(&["**"], &[])],
            ..RUST
        },
    ),
];
const MANIFEST: &str = "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n";
const SOURCES: [(&str, &str); 7] = [
    ("Cargo.toml", MANIFEST),
    ("src/lib.rs", "pub mod core;\n"),
    ("src/core/mod.rs", "mod mesh_tests;\n"),
    ("src/core/mesh_tests.rs", "#[test]\nfn mesh() {}\n"),
    ("src/app.py", "VALUE = 1\n"),
    ("tests/test_b.py", "def test_b() -> None:\n    pass\n"),
    ("tests/unit/test_a.py", "def test_a() -> None:\n    pass\n"),
];
const TESTS: &str = "#[cfg(test)]\nmod tests {\n    #[test]\n    fn runs() {}\n}\n";
const GAME: &str = "native/host/game.rs";
const THING: &str = "native/host/parts/thing.rs";
const MOUNTED_MANIFEST: &str =
    "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n\n[lib]\npath = \"core/lib.rs\"\n";
const NATIVE: &str = "mod host;\n\npub fn size() -> usize {\n    mod local {\n        pub const SIZE: usize = 1;\n    }\n    local::SIZE\n}\n";
const MOUNTED: [(&str, &str); 7] = [
    ("Cargo.toml", MOUNTED_MANIFEST),
    (
        "core/lib.rs",
        "#[path = \"../native/mod.rs\"]\npub mod native;\n",
    ),
    ("native/main.rs", "fn main() {}\n"),
    ("native/mod.rs", NATIVE),
    (
        "native/host/mod.rs",
        "mod game;\n#[path = \"parts/thing.rs\"]\nmod widget;\n",
    ),
    (GAME, TESTS),
    (THING, TESTS),
];
const MODULE_TREE: [Case; 4] = [
    Case {
        groups: &[(
            &["**"],
            &[
                "native::host::game::tests::",
                "native::host::widget::tests::",
            ],
        )],
        ..RUST
    },
    Case {
        groups: &[(&["**"], &["game::tests::", "widget::"])],
        ..RUST
    },
    Case {
        groups: &[(&["**"], &["game::tests::", "host::parts::thing::tests::"])],
        findings: &[(THING, UNSELECTED)],
        ..RUST
    },
    Case {
        groups: &[(&["**"], &["host::world::tests::"])],
        findings: &[(GAME, UNSELECTED), (THING, UNSELECTED)],
        ..RUST
    },
];

fn list(items: &[&str]) -> String {
    let quoted: Vec<String> = items.iter().map(|item| format!("'{item}'")).collect();
    format!("[{}]", quoted.join(", "))
}

fn verify(name: &str, sources: &[(&str, &str)], case: &Case) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let (argv, cwd) = (list(case.argv), case.cwd);
    let commands = format!("test:\n  argv: {argv}\n  accepts_args: true\n  cwd: '{cwd}'\n");
    let group = |(include, targets): &Group| {
        format!("{{include: {}, targets: {}}}", list(include), list(targets))
    };
    let groups: Vec<String> = case.groups.iter().map(group).collect();
    let checks = format!(
        "- id: tests\n  kind: command\n  command: test\n  include: {}\n  skill: {REPAIR}\n  affected: [{}]\n",
        list(case.include),
        groups.join(", ")
    );
    project(&root, &commands, &checks);
    for (path, text) in sources {
        write(&root, path, text);
    }
    let result = invoke(&root, &["config-check"]);
    let findings = case.findings;
    let code = i32::from(!findings.is_empty());
    assert_eq!(result.code, Some(code), "{name}: {}", result.output());
    let reported = result.stderr.matches("ERROR [tests]").count();
    assert_eq!(reported, findings.len(), "{name}: {}", result.stderr);
    for (path, fragment) in findings {
        let start = format!("ERROR [tests] {path}: ");
        let line = result.stderr.split_once(&start).unwrap().1.lines().next();
        let line = line.unwrap();
        assert!(line.contains(fragment), "{name}: {line}");
        assert!(line.contains(REPAIR), "{name}: {line}");
        let rerun = line.split_once("RERUN: ").unwrap().1;
        assert!(rerun.contains("config-check"), "{name}: {line}");
    }
    let valid = result.stdout.contains("Scaffold configuration is valid");
    assert_eq!(valid, findings.is_empty(), "{name}: {}", result.output());
}

#[test]
fn config_check_reports_paths_the_affected_map_leaves_untested() {
    for (name, case) in &CASES {
        verify(name, &SOURCES, case);
    }
}

// DECISION: D144
#[test]
fn cargo_targets_follow_the_declared_module_tree() {
    for (index, case) in MODULE_TREE.iter().enumerate() {
        verify(&format!("module tree {index}"), &MOUNTED, case);
    }
}
