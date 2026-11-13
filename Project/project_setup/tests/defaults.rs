// DECISION: D021
use super::write;
use crate::project_setup::initialization::init;
use crate::shared::testing::invoke;
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    thread,
};
use tempfile::TempDir;

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
            "named-if",
            "if True:\n    pass\n",
            "ready = True\nif ready:\n    pass\n",
        ),
        (
            "parameters",
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
            "named-if",
            "fn example() { if true {} }\n",
            "fn example() { let ready = true; if ready {} }\n",
        ),
        (
            "parameters",
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
const BETA: &str = "def value():\n    return 7\n";
const MARKDOWN: [&str; 3] = ["markdown-size", "markdown-width", "markdown-placement"];

fn initialized(language: &str) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let args = ["--language", language].map(str::to_owned);
    assert_eq!(init(&root, &args).unwrap(), 0);
    (temp, root)
}

fn function(language: &Language, lines: usize) -> String {
    let [head, line, tail] = language.function;
    let body = lines - 1 - tail.lines().count();
    format!("{head}{}{tail}", line.repeat(body))
}

fn cases(language: &Language) -> Vec<(&'static str, String, String)> {
    let size = (
        "function-size",
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

fn lint(root: &Path) -> (Option<i32>, Vec<Value>) {
    let result = invoke(root, &["lint", "--json"]);
    let found = serde_json::from_str(&result.stdout);
    let found = found.unwrap_or_else(|error| panic!("{error}: {}", result.output()));
    (result.code, found)
}

fn text<'a>(item: &'a Value, field: &str) -> &'a str {
    item[field].as_str().unwrap()
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
    let path = format!("Project/example.{}", language.extension);
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
        for path in ["Project/x.rs", "Project/tests/x.rs"] {
            write(&root, path, PANICKING);
        }
        let (_, found) = lint(&root);
        assert!(flagged(&found, "panic-path", "Project/x.rs"), "{found:?}");
        assert!(
            !flagged(&found, "panic-path", "Project/tests/x.rs"),
            "{found:?}"
        );
        let unsafe_test = flagged(&found, "unsafe-justification", "Project/tests/x.rs");
        assert!(unsafe_test, "{found:?}");
    }
}

// DECISION: D182
#[test]
fn shipped_rules_keep_compiler_flags_in_the_rig_list() {
    let (_temp, root) = initialized("rust");
    write(
        &root,
        ".agentrig/rust.yaml",
        "rustflags:\n- --cap-lints\n- allow\n",
    );
    let config = "[build]\nrustflags = [\"--cfg\", \"probe\"]\n";
    write(&root, ".cargo/config.toml", config);
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{found:?}");
    let expected = [
        (
            "allow-justification",
            ".agentrig/rust.yaml",
            "justify-allow",
        ),
        ("cargo-rustflags", ".cargo/config.toml", "move-rustflags"),
    ];
    for (rule, path, skill) in expected {
        let finding = found
            .iter()
            .find(|item| text(item, "rule") == rule && text(item, "path") == path);
        let finding = finding.unwrap_or_else(|| panic!("{rule}: {found:?}"));
        let repair = text(finding, "skill").ends_with(&format!("{skill}/SKILL.md"));
        assert_eq!(
            (text(finding, "level"), repair),
            ("error", true),
            "{finding:?}"
        );
    }
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
    let alpha = "from Project.beta.api import value\n\n\ndef run():\n    return value()\n";
    write(&root, "Project/alpha/api.py", alpha);
    write(&root, "Project/beta/api.py", BETA);
    architecture(&root, "missing");
    for name in ["alpha", "beta"] {
        let contract = format!("purpose: {name}\nfiles:\n  api.py: source\ndirectories: {{}}\n");
        write(
            &root,
            &format!("Project/{name}/architecture.yaml"),
            &contract,
        );
    }
    write(
        &root,
        "Project/beta/api.py",
        &format!("{BETA}import Project.alpha.api\n"),
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
    fs::remove_file(root.join("Ledger/README.md")).unwrap();
    let accepted = invoke(&root, &["lint"]);
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
}
