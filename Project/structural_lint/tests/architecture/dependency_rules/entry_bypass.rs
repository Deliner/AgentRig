// DECISION: D228
use super::measured::measured;
use crate::structural_lint::tests::architecture::{Consumer, Language};
use std::fs;

const RULE: &str = "entry-bypass";

const RUST_LAYERS: [(&str, &str); 12] = [
    ("src/lib.rs", "pub mod cli;\npub mod lint;\n"),
    ("src/cli/mod.rs", "mod help;\nmod run;\n"),
    ("src/cli/help.rs", "pub fn help() -> i32 { 0 }\n"),
    (
        "src/cli/run.rs",
        "pub fn run() -> i32 {\n    crate::lint::reporting::tree::tree()\n        + crate::lint::lint()\n        + super::help::help()\n}\n",
    ),
    (
        "src/lint/mod.rs",
        "pub mod reporting;\nmod architecture;\nmod evaluation;\n#[cfg(test)]\nmod tests;\npub fn lint() -> i32 { 1 }\n",
    ),
    ("src/lint/reporting/mod.rs", "pub mod tree;\n"),
    ("src/lint/reporting/tree.rs", "pub fn tree() -> i32 { 2 }\n"),
    (
        "src/lint/architecture/mod.rs",
        "pub mod graph;\npub fn edges() -> i32 { graph::count() }\n",
    ),
    (
        "src/lint/architecture/graph.rs",
        "pub fn count() -> i32 { 3 }\n",
    ),
    (
        "src/lint/evaluation.rs",
        "pub fn evaluate() -> i32 {\n    super::architecture::graph::count() + super::architecture::edges()\n}\n",
    ),
    ("src/lint/tests/mod.rs", "mod coupling;\n"),
    (
        "src/lint/tests/coupling.rs",
        "#[test]\nfn coupling() {\n    assert_eq!(crate::lint::architecture::graph::count(), crate::lint::architecture::edges());\n}\n",
    ),
];

const PYTHON_LAYERS: [(&str, &str); 10] = [
    ("src/cli/__init__.py", ""),
    ("src/cli/help.py", "def help():\n    return 0\n"),
    (
        "src/cli/run.py",
        "from lint.reporting.tree import tree\nfrom lint import lint\nfrom cli.help import help\n",
    ),
    ("src/lint/__init__.py", "def lint():\n    return 1\n"),
    ("src/lint/reporting/__init__.py", ""),
    ("src/lint/reporting/tree.py", "def tree():\n    return 2\n"),
    (
        "src/lint/architecture/__init__.py",
        "from lint.architecture.graph import count\n",
    ),
    (
        "src/lint/architecture/graph.py",
        "def count():\n    return 3\n",
    ),
    (
        "src/lint/evaluation.py",
        "from lint.architecture.graph import count\nfrom lint.architecture import count\n",
    ),
    (
        "src/lint/tests/test_coupling.py",
        "from lint.architecture.graph import count\nfrom lint.architecture import count\n",
    ),
];

const RUST_SHARED: [(&str, &str); 6] = [
    ("src/lib.rs", "pub mod shared;\npub mod user;\n"),
    ("src/shared/mod.rs", "pub mod paths;\npub mod x;\n"),
    ("src/shared/x.rs", "pub fn x() -> i32 { 1 }\n"),
    (
        "src/shared/paths/mod.rs",
        "pub mod glob_patterns;\npub fn path() -> i32 { 2 }\n",
    ),
    (
        "src/shared/paths/glob_patterns.rs",
        "pub fn glob() -> i32 { 3 }\n",
    ),
    (
        "src/user/mod.rs",
        "pub fn user() -> i32 {\n    crate::shared::x::x()\n        + crate::shared::paths::path()\n        + crate::shared::paths::glob_patterns::glob()\n}\n",
    ),
];

const PYTHON_SHARED: [(&str, &str); 6] = [
    ("src/shared/__init__.py", ""),
    ("src/shared/x.py", "def x():\n    return 1\n"),
    (
        "src/shared/paths/__init__.py",
        "def path():\n    return 2\n",
    ),
    (
        "src/shared/paths/glob_patterns.py",
        "def glob():\n    return 3\n",
    ),
    ("src/user/__init__.py", ""),
    (
        "src/user/run.py",
        "from shared.x import x\nfrom shared.paths import path\nfrom shared.paths.glob_patterns import glob\n",
    ),
];

fn found(consumer: &Consumer) -> Vec<String> {
    let found = consumer.lint().into_iter().filter(|item| item.rule == RULE);
    let mut found: Vec<String> = found
        .map(|item| {
            let line = item.line.unwrap_or_default();
            let named = item
                .message
                .split(';')
                .next()
                .unwrap_or_default()
                .to_owned();
            format!("{path}:{line} {named}", path = item.path)
        })
        .collect();
    found.sort();
    found
}

fn past(source: &str, target: &str, entry: &str) -> String {
    let directory = entry.rsplit_once('/').unwrap().0;
    format!("{source} uses {target} inside {directory} past its entry {entry}")
}

fn group(consumer: &Consumer, directory: &str) {
    let path = consumer.root.join(directory).join("architecture.yaml");
    let contract = fs::read_to_string(&path).unwrap();
    fs::write(path, contract + "group: true\n").unwrap();
}

#[test]
fn a_use_inside_another_directory_goes_through_the_entry_of_the_topmost_one() {
    let cases = [
        (
            Language::Rust,
            &RUST_LAYERS[..],
            ["mod.rs", "rs"],
            [2, 2, 3],
        ),
        (
            Language::Python,
            &PYTHON_LAYERS[..],
            ["__init__.py", "py"],
            [1, 1, 1],
        ),
    ];
    for (language, files, [entry, extension], [run, evaluation, coupling]) in cases {
        let tree = format!("src/lint/reporting/tree.{extension}");
        let graph = format!("src/lint/architecture/graph.{extension}");
        let lint = format!("src/lint/{entry}");
        let architecture = format!("src/lint/architecture/{entry}");
        let test = match language {
            Language::Rust => "src/lint/tests/coupling.rs",
            Language::Python => "src/lint/tests/test_coupling.py",
        };
        let mut expected = vec![
            past(&format!("src/cli/run.{extension}:{run}"), &tree, &lint),
            past(
                &format!("src/lint/evaluation.{extension}:{evaluation}"),
                &graph,
                &architecture,
            ),
            past(&format!("{test}:{coupling}"), &graph, &architecture),
        ];
        expected.sort();
        assert_eq!(found(&measured(language, RULE, files)), expected);
    }
}

#[test]
fn a_group_has_no_boundary_of_its_own_while_each_of_its_directories_has_one() {
    let cases = [
        (
            Language::Rust,
            &RUST_SHARED[..],
            ["mod.rs", "rs", "src/user/mod.rs"],
            2,
        ),
        (
            Language::Python,
            &PYTHON_SHARED[..],
            ["__init__.py", "py", "src/user/run.py"],
            1,
        ),
    ];
    for (language, files, [entry, extension, source], first) in cases {
        let x = format!("src/shared/x.{extension}");
        let paths = format!("src/shared/paths/{entry}");
        let glob = format!("src/shared/paths/glob_patterns.{extension}");
        let shared = format!("src/shared/{entry}");
        let consumer = measured(language, RULE, files);
        let line = |offset: usize| format!("{source}:{}", first + offset);
        let closed = [
            past(&line(0), &x, &shared),
            past(&line(1), &paths, &shared),
            past(&line(2), &glob, &shared),
        ];
        assert_eq!(found(&consumer), closed);
        group(&consumer, "src/shared");
        assert_eq!(found(&consumer), [past(&line(2), &glob, &paths)]);
    }
}

#[test]
fn a_directory_without_an_entry_is_closed_to_every_outside_use() {
    let files = [
        ("src/tools/helper.py", "def assist():\n    return 1\n"),
        ("src/app/__init__.py", "from tools.helper import assist\n"),
    ];
    let found = found(&measured(Language::Python, RULE, &files));
    let closed = "src/app/__init__.py:1 uses src/tools/helper.py inside src/tools, which has no entry mod.rs, lib.rs, main.rs or __init__.py";
    assert_eq!(found, [closed]);
}

#[test]
fn entry_bypass_needs_an_enabled_directory_architecture_rule() {
    let files = [("src/lib.rs", "pub fn run() {}\n")];
    let consumer = measured(Language::Rust, RULE, &files);
    consumer.configure(
        "kind: directory-architecture\n",
        "kind: directory-architecture\n    enabled: false\n",
    );
    consumer.invalid();
}
