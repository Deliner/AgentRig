// DECISION: D017
use super::consumer::{LANGUAGES, LEVEL, SKILL, command, configured, consumer, lint, write};
use serde_json::Value;

const WARNING: &str = r#"level: "warning""#;
const RUST_CONDITIONS: &str = r#"fn run() {
    if ready {}
    if (/* reason */ state.ready) {}
    if ready && enabled {}
    if value > 3 {}
    if ready() {}
    if !ready {}
    if let Some(value) = input {}
    if let Some(value) = input && value > 0 {}
    if crate::READY {}
    // if fake && condition {}
    let text = "if fake && condition {}";
    if factory().ready {}
    if values[0] {}
}
"#;
const PYTHON_CONDITIONS: &str = r#"if ready:
    pass
elif state.ready:
    pass
elif ready and enabled:
    pass
if value > 3:
    pass
if ready():
    pass
if not ready:
    pass
result = yes if (ready) else no
result = yes if ready and enabled else no
items = [x for x in values if x > 0]
# if fake and condition:
text = "if fake and condition:"
if factory().ready:
    pass
"#;
const CONDITIONS: [(&str, &str, [u64; 7]); 2] = [
    ("src/example.rs", RUST_CONDITIONS, [4, 5, 6, 7, 9, 13, 14]),
    (
        "src/example.py",
        PYTHON_CONDITIONS,
        [5, 7, 9, 11, 14, 15, 18],
    ),
];
const MALFORMED: [(&str, &str); 2] = [
    ("src/broken.rs", "fn broken( { if }"),
    ("src/broken.py", "def broken(:\n pass"),
];
const COMMENTS: [(&str, &str, &str, &str, &str); 2] = [
    (
        "src/example.rs",
        "//",
        "// SAFETY: the caller checks bounds",
        "/// Documentation.",
        r#"const TEXT: &str = "// DECISION: D001 data";"#,
    ),
    (
        "src/example.py",
        "#",
        "# type: ignore",
        r#""""Documentation.""""#,
        r##"TEXT = "# DECISION: D001 data""##,
    ),
];
const POLICIES: [&str; 3] = [
    "warning: 1",
    r#"level: "fatal""#,
    "level: \"error\"\nwarning: 1",
];

fn reported(findings: &[Value], level: &str) -> Vec<u64> {
    let lines = findings.iter().map(|item| {
        assert_eq!(item["rule"], "source", "{item}");
        assert_eq!(item["level"], level, "{item}");
        assert!(item["skill"].as_str().unwrap().ends_with(SKILL), "{item}");
        item["line"].as_u64().unwrap()
    });
    lines.collect()
}

// INVARIANT: I015
#[test]
fn language_conditions() {
    for (path, source, lines) in CONDITIONS {
        let config = configured("named-if-condition", LEVEL);
        let (_temp, root) = consumer(&config);
        write(&root, path, source);
        let (code, findings) = lint(&root);
        assert_eq!(code, Some(1), "{path}: {findings:?}");
        assert_eq!(reported(&findings, "error"), lines, "{path}");
        write(&root, "lint.yaml", &config.replace(LEVEL, WARNING));
        let (code, findings) = lint(&root);
        assert_eq!(code, Some(0), "{path}: {findings:?}");
        assert_eq!(reported(&findings, "warning"), lines, "{path}");
    }
}

#[test]
fn malformed_source_blocks() {
    for (path, source) in MALFORMED {
        let (_temp, root) = consumer(&configured("named-if-condition", WARNING));
        write(&root, path, source);
        let (code, findings) = lint(&root);
        assert_eq!(code, Some(1), "{path}: {findings:?}");
        assert_eq!(findings[0]["rule"], "syntax", "{path}");
        assert_eq!(findings[0]["path"], path);
        assert!(findings[0]["line"].as_u64().unwrap() >= 1, "{path}");
    }
}

#[test]
fn language_selection_and_overrides() {
    let overrides = "\n    overrides:\n      - include: [\"src/*.py\"]\n        error: 2\n";
    let config = configured("parameter-count", "warning: 1\nerror: 3") + overrides;
    let (_temp, root) = consumer(&config);
    write(
        &root,
        "src/example.py",
        "def example(one, two, three): pass",
    );
    write(&root, "src/example.js", "not valid Python or Rust");
    let (code, findings) = lint(&root);
    assert_eq!(code, Some(1), "{findings:?}");
    assert_eq!(findings[0]["actual"], 3);
    assert_eq!(findings[0]["limit"], 2);
    let selected = config.replace(LANGUAGES, r#"extensions: [".js"]"#);
    write(&root, "lint.yaml", &selected);
    assert_eq!(lint(&root).0, Some(2));
}

#[test]
fn comment_references_through_cli() {
    for (path, lead, directive, documentation, data) in COMMENTS {
        let (_temp, root) = consumer(&configured("comment-reference", LEVEL));
        let reference = format!("{lead} DECISION: D001");
        let plain = format!("{lead} plain");
        let explained = format!("{reference} because");
        let lines = [
            reference.as_str(),
            plain.as_str(),
            explained.as_str(),
            directive,
            documentation,
            data,
        ];
        write(&root, path, &(lines.join("\n") + "\n"));
        let (code, findings) = lint(&root);
        assert_eq!(code, Some(1), "{path}: {findings:?}");
        let found: Vec<_> = findings
            .iter()
            .map(|item| (item["rule"].as_str(), item["line"].as_u64()))
            .collect();
        assert_eq!(found, [2, 3, 4, 5].map(|line| (Some("source"), Some(line))));
        let text = command(&root, "lint", &[]).stdout;
        let location = format!("{path}:2");
        for expected in [
            "ERROR [source] 4 locations",
            location.as_str(),
            SKILL,
            "RERUN:",
        ] {
            assert!(text.contains(expected), "{expected}: {text}");
        }
        write(&root, path, &format!("{reference}\n{data}\n"));
        assert_eq!(lint(&root), (Some(0), Vec::new()));
    }
}

#[test]
fn shared_parse_error_and_extension_defaults() {
    let conditions = configured("named-if-condition", LEVEL).replace(LANGUAGES, "");
    let rule = &conditions[conditions.find("  - id:").unwrap()..];
    let parameters = rule
        .replace(r#"id: "source""#, r#"id: "parameters""#)
        .replace("named-if-condition", "parameter-count")
        .replace(LEVEL, "error: 1");
    let sources = r#"include: ["src/**"]"#;
    let excluded = format!("{sources}\n    exclude: [\"src/other.js\"]");
    let (_temp, root) = consumer(&(conditions.clone() + &parameters).replace(sources, &excluded));
    write(&root, "src/example.pyi", "def example(one, two): ...");
    write(&root, "src/broken.rs", "fn broken( {");
    write(&root, "src/other.js", "not Rust or Python");
    let (code, findings) = lint(&root);
    assert_eq!(code, Some(1), "{findings:?}");
    let rules: Vec<_> = findings.iter().map(|item| item["rule"].as_str()).collect();
    assert_eq!(rules, [Some("syntax"), Some("parameters")]);
    assert_eq!(findings[1]["actual"], 2);
}

#[test]
fn condition_configuration_is_not_numeric() {
    for policy in POLICIES {
        let (_temp, root) = consumer(&configured("named-if-condition", policy));
        assert_eq!(lint(&root).0, Some(2), "{policy}");
    }
}

// DECISION: D156
#[test]
fn single_junctions_report_each_mixed_expression_at_error_level() {
    let config = configured("boolean-expression", "error: 3\njunctions: \"single\"");
    let (_temp, root) = consumer(&config);
    write(
        &root,
        "src/ready.rs",
        "fn ready() -> bool {\n    a && (b || c)\n}\n",
    );
    write(&root, "src/ready.py", "def ready():\n    return a or b\n");
    let (code, findings) = lint(&root);
    assert_eq!(code, Some(1), "{findings:?}");
    let found: Vec<_> = findings
        .iter()
        .map(|item| {
            (
                item["path"].as_str(),
                item["line"].as_u64(),
                item["level"].as_str(),
            )
        })
        .collect();
    assert_eq!(found, [(Some("src/ready.rs"), Some(2), Some("error"))]);
    let message = findings[0]["message"].as_str().unwrap();
    assert!(message.contains("parentheses do not end it"), "{message}");
    write(
        &root,
        "lint.yaml",
        &config.replace("junctions: \"single\"", ""),
    );
    assert_eq!(lint(&root), (Some(0), Vec::new()));
}
