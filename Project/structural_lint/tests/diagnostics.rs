// DECISION: D016
use super::architecture::FRONTMATTER;
use crate::shared::testing::{Run, git, invoke, launch, replay, run, worker};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const SKILL: &str = ".agents/skills/refactor-large-file/SKILL.md";
const REPAIR: &str = ".agents/skills/repair/SKILL.md";
const EXTENSIONS: &str = r#"extensions: [".rs", ".py", ".ts"]"#;
const CONFIG: &str = r#"version: 1
config_skill: ".agents/skills/repair/SKILL.md"

rules:
  - id: "source"
    kind: "nonblank-lines"
    target: "file"
    include: ["src/**"]
    extensions: [".rs", ".py", ".ts"]
    warning: 3
    error: 5
    warning_skill: ".agents/skills/refactor-large-file/SKILL.md"
    error_skill: ".agents/skills/refactor-large-file/SKILL.md"
"#;
const OVERRIDES: &str = r#"
    overrides:
      - include: ["src/**"]
        extensions: [".rs"]
        warning: 7
        error: 9
      - include: ["src/special.rs"]
        warning: 10
        error: 12
"#;
const INVALID: [(&str, &str); 10] = [
    (r#"kind: "nonblank-lines""#, r#"kind: "unknown""#),
    (r#"target: "file""#, r#"target: "directory""#),
    ("warning: 3", "warning: 5"),
    ("warning: 3", "warning: -1"),
    ("warning: 3", "warn: 3"),
    (r#"include: ["src/**"]"#, r#"include: ["["]"#),
    (r#"include: ["src/**"]"#, "include: []"),
    ("version: 1", "version: 99"),
    (
        r#"error_skill: ".agents/skills/refactor-large-file/SKILL.md""#,
        r#"error_skill: "missing/SKILL.md""#,
    ),
    (
        r#"warning_skill: ".agents/skills/refactor-large-file/SKILL.md""#,
        "",
    ),
];

fn prepare(root: &Path, config: &str) {
    for skill in [SKILL, REPAIR] {
        let path = root.join(skill);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, FRONTMATTER).unwrap();
    }
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("lint.yaml"), config).unwrap();
}

fn consumer(config: &str) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    prepare(&root, config);
    (temp, root)
}

fn write(root: &Path, name: &str, text: &str) {
    fs::write(root.join(name), text).unwrap();
}

fn command(root: &Path, flags: &[&str]) -> Run {
    let mut args = vec!["lint", "--config", "lint.yaml"];
    args.extend(flags);
    run(&mut launch(worker(), root, &args), "")
}

fn parsed(result: &Run) -> Vec<Value> {
    let findings = serde_json::from_str(&result.stdout);
    findings.unwrap_or_else(|error| panic!("{error}: {}", result.output()))
}

fn lint(root: &Path) -> (Option<i32>, Vec<Value>) {
    let result = command(root, &["--json", "--warnings"]);
    (result.code, parsed(&result))
}

fn replayed(finding: &Value) -> (Option<i32>, Vec<Value>) {
    let result = replay(finding["rerun"].as_str().unwrap());
    (result.code, parsed(&result))
}

fn levels(root: &Path) {
    write(root, "src/warned.rs", &"line\n".repeat(4));
    write(root, "src/failed.rs", &"line\n".repeat(6));
}

fn threshold(lines: usize, level: Option<&str>, code: i32) {
    let (_temp, root) = consumer(CONFIG);
    for extension in ["rs", "py", "ts"] {
        let name = format!("src/example.{extension}");
        write(&root, &name, &"line\n\n".repeat(lines));
    }
    fs::write(root.join("src/ignored.bin"), [0xff; 100]).unwrap();
    let (actual, findings) = lint(&root);
    assert_eq!(actual, Some(code), "{lines}: {findings:?}");
    assert_eq!(
        findings.len(),
        level.map_or(0, |_| 3),
        "{lines}: {findings:?}"
    );
    for item in findings {
        assert_eq!(item["level"].as_str(), level, "{item}");
        assert_eq!(item["skill"], SKILL, "{item}");
        assert_eq!(item["actual"], lines, "{item}");
        assert_eq!(item["rule"], "source", "{item}");
    }
}

fn directory() -> String {
    CONFIG
        .replace("nonblank-lines", "directory-entries")
        .replace(r#"target: "file""#, r#"target: "directory""#)
        .replace(r#"include: ["src/**"]"#, r#"include: ["src"]"#)
        .replace(EXTENSIONS, "")
}

fn touch(root: &Path, names: impl IntoIterator<Item = String>) {
    for name in names {
        write(root, &name, "");
    }
}

// INVARIANT: I004
#[test]
fn native_lint_thresholds_and_skills() {
    threshold(3, None, 0);
    threshold(4, Some("warning"), 0);
    threshold(5, Some("warning"), 0);
    threshold(6, Some("error"), 1);
}

// DECISION: D049
#[test]
fn warnings_are_shown_only_on_request() {
    let (_temp, root) = consumer(CONFIG);
    levels(&root);
    let default = command(&root, &[]);
    assert_eq!(default.code, Some(1), "{}", default.output());
    let blocked = "ERROR [source] 1 location: one cause, listed below.";
    assert!(default.stdout.contains(blocked), "{}", default.stdout);
    assert!(
        default.stdout.contains("\n  src/failed.rs: "),
        "{}",
        default.stdout
    );
    assert!(
        !default.output().contains("WARNING"),
        "{}",
        default.output()
    );
    let warned = command(&root, &["--warnings"]).stdout;
    let reported = "WARNING [source] 1 location: one cause, listed below.";
    assert!(warned.contains(reported), "{warned}");
    assert!(warned.contains("\n  src/warned.rs: "), "{warned}");
    fs::remove_file(root.join("src/failed.rs")).unwrap();
    let quiet = command(&root, &["--json"]);
    assert_eq!((quiet.code, quiet.stdout.trim()), (Some(0), "[]"));
}

#[test]
fn rerun_reproduces_findings_with_their_flags() {
    let (_temp, root) = consumer(CONFIG);
    levels(&root);
    let (code, findings) = lint(&root);
    let mut found: Vec<_> = findings.iter().map(|item| item["level"].as_str()).collect();
    found.sort();
    assert_eq!(found, [Some("error"), Some("warning")]);
    for finding in &findings {
        assert_eq!(replayed(finding), (code, findings.clone()));
    }
}

#[test]
fn missing_skill_is_named_in_the_finding() {
    let (_temp, root) = consumer(CONFIG);
    fs::remove_file(root.join(SKILL)).unwrap();
    let (code, findings) = lint(&root);
    assert_eq!(code, Some(2), "{findings:?}");
    assert_eq!(findings[0]["rule"], "configuration");
    let message = findings[0]["message"].as_str().unwrap();
    assert!(
        message.contains(root.join(SKILL).to_str().unwrap()),
        "{message}"
    );
}

#[test]
fn project_mode_reports_a_missing_skill_as_a_json_finding() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let initialized = invoke(&root, &["init"]);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    let skill = root.join(".agentrig/skills/remove-debug/SKILL.md");
    fs::remove_file(&skill).unwrap();
    let skill = skill.to_str().unwrap();
    let result = invoke(&root, &["lint", "--json"]);
    let findings = parsed(&result);
    let rules: Vec<_> = findings.iter().map(|item| item["rule"].as_str()).collect();
    assert_eq!((result.code, rules), (Some(2), vec![Some("configuration")]));
    let message = findings[0]["message"].as_str().unwrap();
    assert!(message.contains(skill), "{message}");
    assert_eq!(replayed(&findings[0]), (Some(2), findings.clone()));
    let text = invoke(&root, &["lint"]);
    assert_eq!((text.code, text.stdout.as_str()), (Some(2), ""));
    assert!(text.stderr.contains("ERROR [lint]"), "{}", text.stderr);
    assert!(text.stderr.contains(skill), "{}", text.stderr);
}

#[test]
fn overrides_and_exclusions() {
    let excluded = format!("{EXTENSIONS}\n    exclude: [\"src/excluded.rs\"]");
    let config = CONFIG.replace(EXTENSIONS, &excluded) + OVERRIDES;
    let (_temp, root) = consumer(&config);
    for name in ["ordinary.rs", "ordinary.py", "special.rs", "excluded.rs"] {
        write(&root, &format!("src/{name}"), &"line\n".repeat(8));
    }
    let (code, findings) = lint(&root);
    assert_eq!(code, Some(1), "{findings:?}");
    let level = |item: &Value| {
        let path = item["path"].as_str().unwrap().to_owned();
        (path, item["level"].as_str().unwrap().to_owned())
    };
    let found: BTreeMap<_, _> = findings.iter().map(level).collect();
    let expected = [("src/ordinary.py", "error"), ("src/ordinary.rs", "warning")];
    let expected = expected.map(|(path, level)| (path.to_owned(), level.to_owned()));
    assert_eq!(found, BTreeMap::from(expected));
}

// INVARIANT: I014
#[test]
fn invalid_config_is_actionable() {
    for (old, new) in INVALID {
        let (_temp, root) = consumer(&CONFIG.replace(old, new));
        let (code, findings) = lint(&root);
        assert_eq!(code, Some(2), "{old} -> {new}: {findings:?}");
        assert_eq!(findings[0]["rule"], "configuration", "{old} -> {new}");
        let skill = findings[0]["skill"].as_str().unwrap();
        assert!(
            skill.ends_with("repair/SKILL.md"),
            "{old} -> {new}: {skill}"
        );
    }
}

#[test]
fn directory_counts_immediate_children() {
    let (_temp, root) = consumer(&directory());
    fs::create_dir(root.join("src/group")).unwrap();
    touch(&root, (0..20).map(|index| format!("src/group/{index}.txt")));
    assert_eq!(lint(&root), (Some(0), Vec::new()));
    touch(&root, (0..5).map(|index| format!("src/{index}.txt")));
    let (code, findings) = lint(&root);
    assert_eq!(code, Some(1), "{findings:?}");
    assert_eq!(findings[0]["actual"], 6);
}

#[test]
fn directory_count_leaves_out_the_architecture_contract() {
    let (_temp, root) = consumer(&directory());
    write(&root, "src/architecture.yaml", "");
    touch(&root, (0..5).map(|index| format!("src/{index}.txt")));
    let (code, findings) = lint(&root);
    assert_eq!(code, Some(0), "{findings:?}");
    let counts: Vec<_> = findings.iter().map(|item| &item["actual"]).collect();
    assert_eq!(counts, [5]);
    write(&root, "src/5.txt", "");
    let (code, findings) = lint(&root);
    assert_eq!(code, Some(1), "{findings:?}");
    assert_eq!(findings[0]["actual"], 6);
}

#[test]
fn staged_files_and_config_are_isolated() {
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();
    let root = base.join("repository");
    prepare(&root, CONFIG);
    git(&root, &["init", "-q"]);
    write(&root, "src/example.rs", &"line\n".repeat(6));
    git(&root, &["add", "."]);
    write(&root, "src/example.rs", "line\n");
    write(&root, "lint.yaml", &CONFIG.replace("error: 5", "error: 50"));
    assert_eq!(lint(&root).0, Some(0));
    let staged = base.join("staged");
    fs::create_dir(&staged).unwrap();
    let prefix = format!("--prefix={}/", staged.display());
    git(&root, &["checkout-index", "--all", &prefix]);
    let (code, findings) = lint(&staged);
    assert_eq!(code, Some(1), "{findings:?}");
    assert_eq!(findings[0]["actual"], 6);
    assert_eq!(findings[0]["limit"], 5);
}

#[test]
fn effective_override_thresholds_must_be_valid() {
    let overrides = "\n    overrides:\n      - include: [\"src/**\"]\n        warning: 9\n";
    let (_temp, root) = consumer(&format!("{CONFIG}{overrides}"));
    write(&root, "src/example.rs", "line");
    assert_eq!(lint(&root).0, Some(2));
}

#[test]
fn directory_rule_rejects_extension_filter() {
    let config = CONFIG
        .replace("nonblank-lines", "directory-entries")
        .replace(r#"target: "file""#, r#"target: "directory""#);
    let (_temp, root) = consumer(&config);
    assert_eq!(lint(&root).0, Some(2));
}
