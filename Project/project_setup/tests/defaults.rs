// DECISION: D021
use super::{initialized, lint, text};
use crate::shared::strict_yaml;
use crate::shared::testing::{invoke, write};
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::Path, thread};

struct Language {
    name: &'static str,
    extension: &'static str,
    function: [&'static str; 3],
    rules: [(&'static str, &'static str, &'static str); 3],
}

const PYTHON: Language = Language {
    name: "python",
    extension: "py",
    function: ["def example():\n", "    value = 0\n", ""],
    rules: [
        (
            "named-if-condition",
            "if True:\n    pass\n",
            "ready = True\nif ready:\n    pass\n",
        ),
        (
            "parameter-count",
            "def example(a, b, c, d, e): pass\n",
            "def example(a, b, c, d): pass\n",
        ),
        (
            "comments",
            "# note\nvalue = 0\n",
            "# DECISION: D001\nvalue = 0\n",
        ),
    ],
};
const RUST: Language = Language {
    name: "rust",
    extension: "rs",
    function: ["fn example() {\n", "    let value = 0;\n", "}\n"],
    rules: [
        (
            "named-if-condition",
            "fn example() { if true {} }\n",
            "fn example() { let ready = true; if ready {} }\n",
        ),
        (
            "parameter-count",
            "fn example(a:i8,b:i16,c:i32,d:i64,e:u8) {}\n",
            "fn example(a:i8,b:i16,c:i32,d:i64) {}\n",
        ),
        (
            "comments",
            "/// note\nfn example() {}\n",
            "// INVARIANT: I001\nfn example() {}\n",
        ),
    ],
};
const FUNCTION_LIMIT: usize = 40;
const PANICKING: &str =
    "fn value() -> u8 {\n    Some(1).unwrap()\n}\nfn raw() {\n    unsafe {}\n}\n";
const LOCKS: [&str; 3] = ["Cargo.lock", "package-lock.json", "uv.lock"];
const INDEXES: [&str; 6] = [
    "Ledger/Decisions.md",
    "Ledger/Invariants.md",
    "Ledger/Plan.md",
    "Ledger/Backlog.md",
    "Ledger/Archive/Plan.md",
    "Ledger/Archive/Backlog.md",
];
const BETA: &str = "def value():\n    return 7\n";
const MARKDOWN: [&str; 3] = ["markdown-size", "markdown-width", "markdown-placement"];

fn function(language: &Language, lines: usize) -> String {
    let [head, line, tail] = language.function;
    let body = lines - 1 - tail.lines().count();
    format!("{head}{}{tail}", line.repeat(body))
}

fn cases(language: &Language) -> Vec<(&'static str, String, String)> {
    let size = (
        "function-lines",
        function(language, FUNCTION_LIMIT + 1),
        function(language, FUNCTION_LIMIT),
    );
    let owned =
        |(rule, invalid, boundary): (_, &str, &str)| (rule, invalid.into(), boundary.into());
    language
        .rules
        .into_iter()
        .map(owned)
        .chain([size])
        .collect()
}

fn flagged(found: &[Value], rule: &str, path: &str) -> bool {
    let matches = |item: &Value| text(item, "rule") == rule && text(item, "path") == path;
    found.iter().any(matches)
}

fn guided(found: &[Value], skill: &str) -> bool {
    found
        .iter()
        .all(|item| text(item, "skill").ends_with(skill))
}

fn blocks(language: &Language, (rule, invalid, boundary): (&str, String, String)) {
    let (_temp, root) = initialized(language.name);
    let path = format!("example.{}", language.extension);
    write(&root, &path, &invalid);
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{rule}: {found:?}");
    let error = |item: &Value| text(item, "rule") == rule && text(item, "level") == "error";
    assert!(found.iter().any(error), "{rule}: {found:?}");
    assert!(guided(&found, "SKILL.md"), "{found:?}");
    let check = invoke(&root, &["check"]);
    assert_eq!(check.code, Some(1), "{rule}: {}", check.output());
    assert!(check.stderr.contains("ERROR [lint]"), "{}", check.output());
    write(&root, &path, &boundary);
    let accepted = invoke(&root, &["lint"]);
    assert_eq!(accepted.code, Some(0), "{rule}: {}", accepted.output());
}

// INVARIANT: I017
#[test]
fn strict_defaults_block_delivery() {
    thread::scope(|scope| {
        let mut handles = Vec::new();
        for language in [&PYTHON, &RUST] {
            for case in cases(language) {
                handles.push(scope.spawn(move || blocks(language, case)));
            }
        }
        for handle in handles {
            handle.join().unwrap();
        }
    });
}

#[test]
fn shipped_rules_skip_test_files_only_where_decided() {
    for language in [PYTHON, RUST] {
        let (_temp, root) = initialized(language.name);
        for path in ["x.rs", "tests/x.rs"] {
            write(&root, path, PANICKING);
        }
        let (_, found) = lint(&root);
        assert!(flagged(&found, "panic-path", "x.rs"), "{found:?}");
        assert!(!flagged(&found, "panic-path", "tests/x.rs"), "{found:?}");
        let unsafe_test = flagged(&found, "unsafe-justification", "tests/x.rs");
        assert!(unsafe_test, "{found:?}");
    }
}

// DECISION: D198
#[test]
fn shipped_program_language_allows_only_the_init_language() {
    for language in ["python", "rust"] {
        let (_temp, root) = initialized(language);
        let config: Value = strict_yaml::read(&root.join(".agentrig/lint.yaml")).unwrap();
        let rules = config["rules"].as_array().unwrap();
        let rule = rules.iter().find(|rule| rule["id"] == "program-language");
        let allowed = &rule.unwrap()["allowed_languages"];
        assert_eq!(allowed, &Value::from(vec![language]), "{rule:?}");
        let checked = invoke(&root, &["config-check"]);
        assert_eq!(checked.code, Some(0), "{}", checked.output());
    }
    let (_temp, root) = initialized("rust");
    write(&root, "tool/x.js", "console.log(1);\n");
    write(&root, "tool/build", "#!/usr/bin/env python3\nprint(1)\n");
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{found:?}");
    let skill = root.join(".agentrig/skills/rewrite-in-allowed-language/SKILL.md");
    assert!(skill.is_file(), "{}", skill.display());
    for (path, language) in [("tool/x.js", "javascript"), ("tool/build", "python")] {
        let reported =
            |item: &&Value| text(item, "rule") == "program-language" && text(item, "path") == path;
        let finding = found.iter().find(reported);
        let finding = finding.unwrap_or_else(|| panic!("{path}: {found:?}"));
        let message = text(finding, "message");
        let named = message.contains(language) && message.contains("allowed languages: rust;");
        assert!(named, "{message}");
        let repair = Path::new(text(finding, "skill")) == skill;
        let outcome = (text(finding, "level"), repair);
        assert_eq!(outcome, ("error", true), "{finding:?}");
    }
    let check = invoke(&root, &["check"]);
    assert_eq!(check.code, Some(1), "{}", check.output());
    assert!(check.stderr.contains("ERROR [lint]"), "{}", check.output());
    fs::remove_file(root.join("tool/x.js")).unwrap();
    fs::remove_file(root.join("tool/build")).unwrap();
    fs::remove_dir(root.join("tool")).unwrap();
    let accepted = invoke(&root, &["lint"]);
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
}

#[test]
fn shipped_file_size_skips_package_manager_lock_files() {
    let (_temp, root) = initialized("rust");
    let content = "fn f() {}\n".repeat(600);
    for name in LOCKS.into_iter().chain(["large.rs"]) {
        write(&root, &format!("Project/tool/{name}"), &content);
    }
    let (_, found) = lint(&root);
    assert!(
        flagged(&found, "file-size", "Project/tool/large.rs"),
        "{found:?}"
    );
    for name in LOCKS {
        let path = format!("Project/tool/{name}");
        assert!(!flagged(&found, "file-size", &path), "{found:?}");
    }
}

// DECISION: D237
#[test]
fn shipped_file_size_skips_the_memory_indexes_but_not_their_cards() {
    let (_temp, root) = initialized("rust");
    let content = "Line.\n".repeat(600);
    let card = "Ledger/Plan/001.md";
    for path in INDEXES.into_iter().chain([card]) {
        write(&root, path, &content);
    }
    let (_, found) = lint(&root);
    assert!(flagged(&found, "file-size", card), "{found:?}");
    for path in INDEXES {
        assert!(!flagged(&found, "file-size", path), "{found:?}");
    }
}

fn architecture(root: &Path, expected: &str) {
    let (code, found) = lint(root);
    assert_eq!(code, Some(1), "{found:?}");
    assert!(
        guided(&found, "refactor-large-directory/SKILL.md"),
        "{found:?}"
    );
    let reported = |item: &Value| {
        text(item, "rule") == "architecture" && text(item, "message").contains(expected)
    };
    assert!(found.iter().any(reported), "{expected}: {found:?}");
}

#[test]
fn shipped_architecture_reports_contract_and_cycle() {
    let (_temp, root) = initialized("python");
    let alpha = "from Project.beta import value\n\n\ndef run():\n    return value()\n";
    write(&root, "Project/alpha/__init__.py", alpha);
    write(&root, "Project/beta/__init__.py", BETA);
    architecture(&root, "missing");
    for name in ["alpha", "beta"] {
        let contract =
            format!("purpose: {name}\nfiles:\n  __init__.py: entry\ndirectories: {{}}\n");
        write(
            &root,
            &format!("Project/{name}/architecture.yaml"),
            &contract,
        );
    }
    write(
        &root,
        "Project/beta/__init__.py",
        &format!("{BETA}import Project.alpha\n"),
    );
    architecture(&root, "cycle");
}

#[test]
fn shipped_markdown_bounds() {
    let (_temp, root) = initialized("python");
    let skill = root.join(".agentrig/skills/condense-markdown/SKILL.md");
    assert!(skill.is_file(), "{}", skill.display());
    let prose = "a line of prose\n".repeat(75);
    write(
        &root,
        "README.md",
        &format!("# Title\n\n{prose}{}\n", "x".repeat(130)),
    );
    write(&root, "Project/notes.md", "stray content\n");
    write(
        &root,
        "Ledger/README.md",
        &format!("# Big\n\n{}", "a line\n".repeat(75)),
    );
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{found:?}");
    let markdown = |item: &&Value| text(item, "rule").starts_with("markdown-");
    let found: Vec<Value> = found.iter().filter(markdown).cloned().collect();
    let rules: BTreeSet<&str> = found.iter().map(|item| text(item, "rule")).collect();
    assert_eq!(rules, BTreeSet::from(MARKDOWN), "{found:?}");
    assert!(guided(&found, "condense-markdown/SKILL.md"), "{found:?}");
    let memory = |item: &Value| text(item, "path").starts_with("Ledger");
    assert!(!found.iter().any(memory), "{found:?}");
    write(
        &root,
        "README.md",
        &format!("# Title\n\n{}", "a line\n".repeat(60)),
    );
    fs::remove_file(root.join("Project/notes.md")).unwrap();
    fs::remove_dir(root.join("Project")).unwrap();
    fs::remove_file(root.join("Ledger/README.md")).unwrap();
    let accepted = invoke(&root, &["lint"]);
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
}

// DECISION: D192
#[test]
fn shipped_state_bounds() {
    let (_temp, root) = initialized("python");
    let state = |lines: usize, width: usize| {
        let prose = "a line of the current VAC\n".repeat(lines - 2);
        format!("# State\n\n{prose}\n{}\n", "x".repeat(width))
    };
    write(&root, "Ledger/State.md", &state(101, 121));
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{found:?}");
    let of_state = |item: &&Value| text(item, "path") == "Ledger/State.md";
    let found: Vec<Value> = found.iter().filter(of_state).cloned().collect();
    let rules: BTreeSet<&str> = found.iter().map(|item| text(item, "rule")).collect();
    let expected = BTreeSet::from(["state-size", "state-width"]);
    assert_eq!(rules, expected, "{found:?}");
    let skill = root.join(".agentrig/skills/edit-state/SKILL.md");
    let named = |item: &Value| Path::new(text(item, "skill")) == skill;
    assert!(found.iter().all(named), "{found:?}");
    let check = invoke(&root, &["check"]);
    assert_eq!(check.code, Some(1), "{}", check.output());
    assert!(check.stderr.contains("ERROR [lint]"), "{}", check.output());
    write(&root, "Ledger/State.md", &state(100, 120));
    let accepted = invoke(&root, &["lint"]);
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
}

struct Bounded {
    path: &'static str,
    header: &'static str,
    lines: usize,
    skill: &'static str,
    rules: [&'static str; 2],
}

const BOUNDED: [Bounded; 2] = [
    Bounded {
        path: "Ledger/NorthStar.md",
        header: "# North Star",
        lines: 30,
        skill: "propose-protected-change",
        rules: ["north-star-size", "north-star-width"],
    },
    Bounded {
        path: "Ledger/Glossary.md",
        header: "# Glossary",
        lines: 100,
        skill: "edit-glossary",
        rules: ["glossary-size", "glossary-width"],
    },
];

fn filled(file: &Bounded, lines: usize, width: usize) -> String {
    let prose = "a line of meaning\n".repeat(lines - 2);
    format!("{}\n\n{prose}\n{}\n", file.header, "x".repeat(width))
}

fn refused(found: &[Value], file: &Bounded) {
    let of_file = |item: &&Value| text(item, "path") == file.path;
    let found: Vec<Value> = found.iter().filter(of_file).cloned().collect();
    let rules: BTreeSet<&str> = found.iter().map(|item| text(item, "rule")).collect();
    assert_eq!(rules, BTreeSet::from(file.rules), "{found:?}");
    let skill = format!("{}/SKILL.md", file.skill);
    assert!(guided(&found, &skill), "{found:?}");
    for limit in [file.lines, 120] {
        let exceeds = format!("exceeds {limit}");
        let message = |item: &Value| text(item, "message").ends_with(&exceeds);
        assert!(found.iter().any(message), "{exceeds}: {found:?}");
    }
}

#[test]
fn shipped_north_star_and_glossary_are_headed_and_bounded() {
    let (_temp, root) = initialized("python");
    for file in &BOUNDED {
        let created = fs::read_to_string(root.join(file.path)).unwrap();
        assert_eq!(created, format!("{}\n", file.header), "{}", file.path);
        write(&root, file.path, &filled(file, file.lines + 1, 121));
    }
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{found:?}");
    for file in &BOUNDED {
        refused(&found, file);
        write(&root, file.path, &filled(file, file.lines, 120));
    }
    let accepted = invoke(&root, &["lint"]);
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
}
