use super::{Consumer, Language, SKILL, binaries, described};
use crate::shared::testing::{git, launch, run, worker};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, os::unix::fs::symlink};

const CYCLE: &str =
    "src/a -> src/b -> src/a; src/a/api.py:1 uses src/b/api.py; src/b/api.py:3 uses src/a/api.py";

fn lint(consumer: &Consumer) -> (Option<i32>, Value) {
    let [integrated, standalone] = binaries().map(|binary| consumer.process(&binary));
    assert_eq!(integrated, standalone);
    integrated
}

fn reported(consumer: &Consumer, violation: &str) {
    let (code, findings) = lint(consumer);
    assert_eq!(code, Some(1), "{findings}");
    let items = findings.as_array().unwrap();
    let named = items.iter().any(|item| {
        let message = item["message"].as_str().unwrap();
        message.contains(violation)
    });
    let shaped = items.iter().all(|item| {
        let attributed = item["rule"] == "architecture" && item["skill"] == SKILL;
        attributed && item["actual"].is_null() && item["limit"].is_null()
    });
    assert!(named && shaped, "{findings}");
}

fn explain(consumer: &Consumer, path: &str) -> Value {
    let args = ["lint-explain", path, "--config", "lint.yaml", "--json"];
    let result = run(&mut launch(worker(), &consumer.root, &args), "");
    assert_eq!(result.code, Some(0), "{}", result.output());
    serde_json::from_str(&result.stdout).unwrap()
}

fn keys(value: &Value) -> BTreeSet<&str> {
    match value {
        Value::Object(map) => map.keys().map(String::as_str).collect(),
        Value::Array(items) => items.iter().filter_map(Value::as_str).collect(),
        _ => BTreeSet::new(),
    }
}

#[test]
fn both_lint_binaries_report_the_same_architecture_contracts() {
    for language in [Language::Rust, Language::Python] {
        assert_eq!(lint(&Consumer::new(language)), (Some(0), json!([])));
        let missing = Consumer::new(language);
        fs::remove_file(missing.root.join("src/a/architecture.yaml")).unwrap();
        reported(&missing, "missing");
        let cycle = Consumer::new(language);
        cycle.back_edge();
        reported(&cycle, "cycle");
    }
}

// DECISION: D142
#[test]
fn architecture_warning_level_and_nonmutation() {
    let consumer = Consumer::new(Language::Python);
    consumer.back_edge();
    consumer.configure("level: error", "level: warning");
    let before = consumer.snapshot();
    let (code, findings) = lint(&consumer);
    assert_eq!(code, Some(0), "{findings}");
    assert_eq!(findings.as_array().unwrap().len(), 1, "{findings}");
    let finding = &findings[0];
    assert_eq!(finding["level"], "warning");
    assert_eq!(
        finding["message"],
        format!("directory dependency cycle: {CYCLE}")
    );
    assert_eq!(
        (&finding["path"], &finding["line"]),
        (&json!("src/b/api.py"), &json!(3))
    );
    assert!(consumer.snapshot() == before, "lint changed the project");
    assert_eq!(lint(&consumer), (code, findings));
}

#[test]
fn architecture_discovery_and_directory_explanation() {
    let row = described();
    assert_eq!(keys(&row["languages"]), BTreeSet::from(["rust", "python"]));
    assert_eq!(
        (&row["target"], &row["parameters"]["type"]),
        (&json!("directory"), &json!("policy"))
    );
    let resolution = &row["architecture"]["resolution"];
    assert_eq!(keys(resolution), BTreeSet::from(["rust", "python"]));
    let consumer = Consumer::new(Language::Python);
    let selected = &explain(&consumer, "src/a")["rules"][0];
    assert_eq!(selected["selected"], true);
    assert_eq!(selected["extensions"], json!([".py"]));
    assert_eq!(selected["architecture"]["python_root"], "src");
    consumer.write("src/docs/guide.md", "No source in this directory.\n");
    assert_eq!(explain(&consumer, "src/docs")["rules"][0]["selected"], true);
    let found = consumer.lint();
    let blocking = found.iter().any(|item| item.level.blocking());
    let contract = found
        .iter()
        .any(|item| item.path == "src/docs/architecture.yaml");
    assert!(blocking && contract, "{:?}", super::messages(&found));
    consumer.contract("src/docs");
    consumer.contract("src");
    consumer.passes();
}

#[test]
fn architecture_rejects_git_target_through_escaping_symlink() {
    let consumer = Consumer::new(Language::Python);
    git(&consumer.root, &["init", "-q"]);
    git(&consumer.root, &["add", "src"]);
    let outside = consumer.root.parent().unwrap().join("outside");
    fs::rename(consumer.root.join("src/b"), &outside).unwrap();
    symlink(&outside, consumer.root.join("src/b")).unwrap();
    consumer.configure("version: 1\n", "exclude: ['src/b/**']\nversion: 1\n");
    let (code, findings) = consumer.process(worker());
    assert_eq!(code, Some(1), "{findings}");
    let escaped = findings.as_array().unwrap().iter().any(|item| {
        let message = item["message"].as_str().unwrap();
        message.contains("target escapes project root") && item["line"] == 1
    });
    assert!(escaped, "{findings}");
}
