// DECISION: D018
use super::consumer::{CONFIG, LANGUAGES, LEVEL, check, configured, consumer, lint, write};

const SOURCES: &str = r#"include: ["src/**"]"#;
const PAIR: &str = r#"pairs: [{open: "begin", close: "commit"}]"#;
const SCHEMA: [(&str, &str); 6] = [
    (r#"kind: "nonblank-lines""#, r#"kind: "unknown""#),
    (r#"target: "file""#, r#"target: "directory""#),
    ("warning: 3", "warning: 5"),
    (SOURCES, r#"include: ["["]"#),
    (
        r#"error_skill: ".agents/skills/refactor-large-file/SKILL.md""#,
        r#"error_skill: "missing/SKILL.md""#,
    ),
    ("version: 1", "version: ["),
];
const MISPLACED: [(&str, &str, &str); 5] = [
    (
        "named-if-condition",
        r#"junctions: "single""#,
        "junctions apply only to boolean-expression",
    ),
    ("named-if-condition", PAIR, "pairs require paired-calls"),
    ("paired-calls", "", "paired-calls needs pairs"),
    ("paired-calls", "pairs: []", "pairs need distinct nonempty"),
    (
        "paired-calls",
        r#"pairs: [{open: "lock", close: "lock"}]"#,
        "pairs need distinct nonempty",
    ),
];

fn rust(kind: &str, settings: &str) -> String {
    configured(kind, settings).replace(LANGUAGES, r#"extensions: [".rs"]"#)
}

fn unsupported(selector: &str) {
    let config = configured("named-if-condition", LEVEL).replace(LANGUAGES, "");
    let include = selector.starts_with("include");
    let config = if include {
        config.replace(SOURCES, selector)
    } else {
        format!("{config}    {selector}\n")
    };
    let (_temp, root) = consumer(&config);
    write(&root, "src/run.sh", "if true; then echo ok; fi");
    for (code, findings) in [check(&root), lint(&root)] {
        assert_eq!(code, Some(2), "{selector}: {findings:?}");
        let message = findings[0]["message"].as_str().unwrap();
        let named = ["Rust", "Python", ".sh"]
            .iter()
            .all(|part| message.contains(part));
        assert!(named, "{selector}: {message}");
        let skill = findings[0]["skill"].as_str().unwrap();
        assert!(skill.ends_with("repair/SKILL.md"), "{selector}: {skill}");
    }
}

// INVARIANT: I016
#[test]
fn config_rejects_unsupported_language() {
    for selector in [
        r#"extensions: [".sh"]"#,
        r#"include: ["src/*.sh"]"#,
        SOURCES,
    ] {
        unsupported(selector);
    }
}

#[test]
fn config_check_does_not_parse_source() {
    let (_temp, root) = consumer(&configured("named-if-condition", LEVEL));
    write(&root, "src/broken.rs", "fn broken( {");
    assert_eq!(check(&root), (Some(0), Vec::new()));
    assert_eq!(lint(&root).0, Some(1));
}

#[test]
fn config_check_reports_schema_errors() {
    for (old, new) in SCHEMA {
        let (_temp, root) = consumer(&CONFIG.replace(old, new));
        let (code, findings) = check(&root);
        assert_eq!(code, Some(2), "{old} -> {new}: {findings:?}");
        assert_eq!(findings[0]["rule"], "configuration", "{old} -> {new}");
    }
}

// DECISION: D156
#[test]
fn config_check_rejects_misplaced_rule_settings() {
    for (kind, settings, expected) in MISPLACED {
        let (_temp, root) = consumer(&rust(kind, &format!("{LEVEL}\n{settings}")));
        let (code, findings) = check(&root);
        assert_eq!(code, Some(2), "{kind} {settings}: {findings:?}");
        let message = findings[0]["message"].as_str().unwrap();
        assert!(message.contains(expected), "{expected}: {message}");
    }
}

#[test]
fn config_check_accepts_settings_on_their_rule_and_pairs_once() {
    let (_temp, root) = consumer(&rust(
        "boolean-expression",
        "error: 3\njunctions: \"single\"",
    ));
    assert_eq!(check(&root), (Some(0), Vec::new()));
    let config = rust("paired-calls", &format!("{LEVEL}\n{PAIR}"));
    let (_pairs, root) = consumer(&config);
    assert_eq!(check(&root), (Some(0), Vec::new()));
    let (_, rule) = config.split_at(config.find("  - id:").unwrap());
    let second = rule.replace(r#"id: "source""#, r#"id: "second""#);
    write(&root, "lint.yaml", &format!("{config}{second}"));
    let (code, findings) = check(&root);
    assert_eq!(code, Some(2), "{findings:?}");
    let message = findings[0]["message"].as_str().unwrap();
    assert!(
        message.contains("at most one rule lists pairs"),
        "{message}"
    );
}

#[test]
fn config_check_effective_override() {
    let overrides = "\n    overrides:\n      - include: [\"src/**\"]\n        warning: 9\n";
    let (_temp, root) = consumer(&format!("{CONFIG}{overrides}"));
    write(&root, "src/file.py", "");
    assert_eq!(check(&root).0, Some(2));
}

#[test]
fn disabled_rule_preserves_schema_validation() {
    let config = configured("named-if-condition", LEVEL).replace(LANGUAGES, "enabled: false");
    let (_temp, root) = consumer(&config);
    write(&root, "src/run.sh", "echo ok");
    assert_eq!(check(&root), (Some(0), Vec::new()));
    assert_eq!(lint(&root), (Some(0), Vec::new()));
    write(
        &root,
        "lint.yaml",
        &config.replace(LEVEL, r#"level: "fatal""#),
    );
    assert_eq!(check(&root).0, Some(2));
}

#[test]
fn explicit_suffix_rejected_without_files() {
    let config = configured("named-if-condition", LEVEL);
    let (_temp, root) = consumer(&config.replace(SOURCES, r#"include: ["future/*.sh"]"#));
    assert_eq!(check(&root).0, Some(2));
}
