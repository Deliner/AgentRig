#[path = "../../../testing/mod.rs"]
mod support;
use std::fs;
use support::Fixture;

#[test]
fn repeated_review_requires_explanations_for_new_unchanged_findings() {
    let fixture = Fixture::new("pass");
    let first = fixture.run(None);
    let previous = fixture.0.path().join(format!(
        "reports/{}.json",
        first["run_id"].as_str().unwrap()
    ));
    fixture.mode("fail");
    let rejected = fixture.run(Some(&previous));
    assert_eq!(rejected["verdict"], "BLOCKED");
    assert!(
        rejected["roles"][0]["technical_error"]
            .as_str()
            .unwrap()
            .contains("late finding")
    );
    fixture.mode("late");
    let accepted = fixture.run(Some(&previous));
    assert_eq!(accepted["verdict"], "FAIL");
    assert_eq!(accepted["repair_diff"], "");
}

#[test]
fn repairs_keep_original_base_and_recheck_closed_requirements() {
    let fixture = Fixture::new("fail");
    let first = fixture.run(None);
    let root = fixture.0.path();
    let previous = root.join(format!(
        "reports/{}.json",
        first["run_id"].as_str().unwrap()
    ));
    std::fs::write(root.join("repo/src/value.py"), "value = 2\n").unwrap();
    support::git(&root.join("repo"), &["add", "."]);
    support::git(
        &root.join("repo"),
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-qm",
            "repair",
        ],
    );
    fixture.mode("pass");
    let repaired = fixture.run(Some(&previous));
    assert_eq!(repaired["verdict"], "PASS");
    assert_eq!(repaired["snapshot"]["base"], first["snapshot"]["base"]);
    assert!(
        repaired["repair_diff"]
            .as_str()
            .unwrap()
            .contains("+value = 2")
    );
    assert_ne!(
        repaired["snapshot"]["candidate"],
        first["snapshot"]["candidate"]
    );
}

#[test]
fn late_blocked_checks_require_the_same_omission_explanation() {
    let fixture = Fixture::new("pass");
    let first = fixture.run(None);
    let previous = fixture.0.path().join(format!(
        "reports/{}.json",
        first["run_id"].as_str().unwrap()
    ));
    fixture.mode("blocked");
    let rejected = fixture.run(Some(&previous));
    assert!(
        rejected["roles"][0]["technical_error"]
            .as_str()
            .unwrap()
            .contains("late finding")
    );
    fixture.mode("late-blocked");
    let accepted = fixture.run(Some(&previous));
    assert_eq!(accepted["verdict"], "BLOCKED");
    assert!(accepted["roles"][0]["technical_error"].is_null());
}

#[test]
fn changed_prompts_require_a_new_review_boundary() {
    let fixture = Fixture::new("pass");
    let previous = baseline(&fixture);
    fs::write(
        fixture.0.path().join("prompt.md"),
        "Assess C-1 using revised criteria.",
    )
    .unwrap();
    let rejected = fixture.run(Some(&previous));
    assert_boundary(&rejected, "prompts");
    assert_eq!(fixture.run(None)["verdict"], "PASS");
}

#[test]
fn changed_repository_scope_requires_a_new_review_boundary() {
    for field in ["visible_paths", "contract_paths"] {
        let fixture = Fixture::new("pass");
        let previous = baseline(&fixture);
        scope(&fixture, field, &["src/value.py"]);
        let rejected = fixture.run(Some(&previous));
        assert_boundary(&rejected, "scope");
        assert_eq!(fixture.run(None)["verdict"], "PASS");
    }
}

#[test]
fn changed_normative_files_require_a_new_review_boundary() {
    let fixture = Fixture::new("pass");
    scope(&fixture, "contract_paths", &["src/value.py"]);
    let previous = baseline(&fixture);
    let repo = fixture.0.path().join("repo");
    fs::write(repo.join("src/value.py"), "value = 2\n").unwrap();
    support::git(&repo, &["add", "."]);
    support::git(
        &repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-qm",
            "new criteria",
        ],
    );
    let rejected = fixture.run(Some(&previous));
    assert_boundary(&rejected, "normative");
    assert_eq!(fixture.run(None)["verdict"], "PASS");
}

#[test]
fn model_changes_and_reordered_scope_preserve_review_comparability() {
    let fixture = Fixture::new("pass");
    scope(&fixture, "visible_paths", &["src/**", "optional/**"]);
    let previous = baseline(&fixture);
    scope(&fixture, "visible_paths", &["optional/**", "src/**"]);
    let path = fixture.0.path().join("config.yaml");
    let config = fs::read_to_string(&path)
        .unwrap()
        .replace("model: test", "model: replacement")
        .replace("timeout_seconds: 3", "timeout_seconds: 5");
    fs::write(path, config).unwrap();
    assert_eq!(fixture.run(Some(&previous))["verdict"], "PASS");
}

fn baseline(fixture: &Fixture) -> std::path::PathBuf {
    let report = fixture.run(None);
    assert_eq!(report["verdict"], "PASS");
    fixture.0.path().join(format!(
        "reports/{}.json",
        report["run_id"].as_str().unwrap()
    ))
}

fn scope(fixture: &Fixture, field: &str, paths: &[&str]) {
    let path = fixture.0.path().join("project.yaml");
    let mut project: serde_json::Value = review_runner::config::yaml::read(&path).unwrap();
    project["repository"][field] = serde_json::json!(paths);
    fs::write(path, review_runner::config::yaml::encode(&project).unwrap()).unwrap();
}

fn assert_boundary(report: &serde_json::Value, cause: &str) {
    assert_eq!(report["verdict"], "BLOCKED");
    assert!(report["roles"].as_array().unwrap().is_empty());
    let error = report["technical_error"].as_str().unwrap();
    assert!(error.contains(cause), "{error}");
    assert!(error.contains("new review boundary"), "{error}");
}
