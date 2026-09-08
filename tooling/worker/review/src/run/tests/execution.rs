#[path = "../../../testing/mod.rs"]
mod support;
use std::{fs, os::unix::fs::PermissionsExt, process::Command};
use support::Fixture;

fn hg(root: &std::path::Path, args: &[&str]) {
    let output = std::process::Command::new("hg")
        .current_dir(root)
        .env("HGPLAIN", "1")
        .env("HGRCPATH", "")
        .env("HGRCSKIPREPO", "1")
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn mercurial_fixture(external: bool) -> Fixture {
    let fixture = Fixture::new("fail");
    let root = fixture.0.path();
    let repo = root.join("repo");
    std::fs::remove_dir_all(repo.join(".git")).unwrap();
    hg(&repo, &["init"]);
    hg(&repo, &["add"]);
    hg(&repo, &["commit", "-m", "base", "-u", "Test"]);
    let config = root.join("project.yaml");
    let yaml = std::fs::read_to_string(&config)
        .unwrap()
        .replace("repository:\n", "repository:\n  vcs: mercurial\n");
    std::fs::write(config, yaml).unwrap();
    if external {
        let path = root.join("project.yaml");
        let mut project: serde_json::Value = review_runner::config::yaml::read(&path).unwrap();
        project["repository"]["vcs"] = serde_json::json!({"command": [
            "python3", "-B", std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
                .join("../examples/external_vcs.py")
        ]});
        std::fs::write(path, review_runner::config::yaml::encode(&project).unwrap()).unwrap();
    }
    fixture
}

#[test]
fn mercurial_review_and_repair_preserve_revision_scope_and_workspace() {
    review_and_repair(false);
}

#[test]
fn external_review_and_repair_preserve_revision_scope_and_workspace() {
    review_and_repair(true);
}

fn review_and_repair(external: bool) {
    let fixture = mercurial_fixture(external);
    let root = fixture.0.path();
    let repo = root.join("repo");
    let first = fixture.run(None);
    assert_eq!(first["verdict"], "FAIL", "{first:#}");
    let previous = root.join(format!(
        "reports/{}.json",
        first["run_id"].as_str().unwrap()
    ));
    std::fs::write(repo.join("src/value.py"), "value = 2\n").unwrap();
    hg(&repo, &["commit", "-m", "repair", "-u", "Test"]);
    std::fs::write(repo.join("src/value.py"), "uncommitted\n").unwrap();
    fixture.mode("pass");
    let repaired = fixture.run(Some(&previous));
    assert_eq!(repaired["verdict"], "PASS", "{repaired:#}");
    assert_eq!(repaired["snapshot"]["base"], first["snapshot"]["base"]);
    assert_ne!(
        repaired["snapshot"]["candidate"],
        first["snapshot"]["candidate"]
    );
    assert!(
        repaired["repair_diff"]
            .as_str()
            .unwrap()
            .contains("+value = 2")
    );
    assert_eq!(
        std::fs::read_to_string(repo.join("src/value.py")).unwrap(),
        "uncommitted\n"
    );
}

#[test]
fn isolated_parallel_reviews_persist_exact_answers_and_cleanup() {
    let fixture = Fixture::new("parallel");
    let report = fixture.run(None);
    assert_eq!(report["verdict"], "PASS");
    assert_eq!(report["roles"].as_array().unwrap().len(), 2);
    for role in report["roles"].as_array().unwrap() {
        assert_eq!(role["format_attempts"], 1);
        assert!(
            role["raw_response"]
                .as_str()
                .unwrap()
                .contains("Informational")
        );
    }
    let times: Vec<Vec<f64>> = report["roles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|role| {
            role["cli_events"]
                .as_str()
                .unwrap()
                .lines()
                .map(|line| line.parse().unwrap())
                .collect()
        })
        .collect();
    assert!(
        times[0][0] < times[1][1] && times[1][0] < times[0][1],
        "reviewers must overlap"
    );
    let id = report["run_id"].as_str().unwrap();
    assert!(fixture.0.path().join(format!("reports/{id}.md")).is_file());
    assert_ne!(fixture.run(None)["run_id"], id);
}

#[test]
fn repeated_external_review_rejects_a_changed_backend() {
    let fixture = mercurial_fixture(true);
    let first = fixture.run(None);
    let root = fixture.0.path();
    let previous = root.join(format!(
        "reports/{}.json",
        first["run_id"].as_str().unwrap()
    ));
    let path = root.join("project.yaml");
    let mut project: serde_json::Value = review_runner::config::yaml::read(&path).unwrap();
    project["repository"]["vcs"] = serde_json::json!("mercurial");
    std::fs::write(path, review_runner::config::yaml::encode(&project).unwrap()).unwrap();
    let denied = fixture.run(Some(&previous));
    assert_eq!(denied["verdict"], "BLOCKED");
    assert!(
        denied["technical_error"]
            .as_str()
            .unwrap()
            .contains("another VCS source")
    );
}

#[test]
fn missing_repository_root_is_preserved_as_a_technical_report() {
    let fixture = Fixture::new("pass");
    let root = fixture.0.path();
    let report = review_runner::run::run(
        &root.join("config.yaml"),
        review_runner::run::Request {
            root: root.join("missing"),
            base: "HEAD".into(),
            candidate: "HEAD".into(),
            previous_report: None,
            tool: "review_code".into(),
        },
    )
    .unwrap();
    assert_eq!(report.verdict, "BLOCKED");
    assert!(report.technical_error.is_some() && report.roles.is_empty());
    assert!(
        root.join(format!("reports/{}.json", report.run_id))
            .exists()
    );
}

#[test]
fn repeated_external_review_rejects_another_repository_with_the_same_ids() {
    let first = mercurial_fixture(true);
    let report = first.run(None);
    let root = first.0.path();
    let previous = root.join(format!(
        "reports/{}.json",
        report["run_id"].as_str().unwrap()
    ));
    let other = mercurial_fixture(true);
    std::fs::remove_dir_all(other.0.path().join("repo")).unwrap();
    hg(
        other.0.path(),
        &["clone", root.join("repo").to_str().unwrap(), "repo"],
    );
    let denied = other.run(Some(&previous));
    assert_eq!(denied["snapshot"]["base"], report["snapshot"]["base"]);
    assert_eq!(denied["verdict"], "BLOCKED");
    assert!(
        denied["technical_error"]
            .as_str()
            .unwrap()
            .contains("another VCS source")
    );
}
#[test]
fn exhaustion_cannot_be_overridden_by_a_later_valid_response() {
    let report = Fixture::new("exhausted").run(None);
    assert_eq!(report["verdict"], "BLOCKED");
    for role in report["roles"].as_array().unwrap() {
        assert!(
            role["technical_error"]
                .as_str()
                .unwrap()
                .contains("format attempt limit")
        );
        assert!(role["raw_response"].is_string());
    }
}
#[test]
fn timeout_is_a_reported_technical_failure() {
    let report = Fixture::new("timeout").run(None);
    assert_eq!(report["verdict"], "BLOCKED");
    assert!(
        report["roles"][0]["technical_error"]
            .as_str()
            .unwrap()
            .contains("timeout")
    );
}

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
fn scope_rejection_is_persisted_before_any_model_runs() {
    let fixture = Fixture::new("pass");
    let root = fixture.0.path().join("repo");
    std::fs::write(root.join("forbidden.txt"), "private").unwrap();
    support::git(&root, &["add", "."]);
    support::git(
        &root,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-qm",
            "forbidden",
        ],
    );
    let report = fixture.run(None);
    assert_eq!(report["verdict"], "BLOCKED");
    assert!(report["roles"].as_array().unwrap().is_empty());
    assert!(
        report["technical_error"]
            .as_str()
            .unwrap()
            .contains("forbidden.txt")
    );
    assert_eq!(report["request"]["candidate"], "HEAD");
}

#[test]
fn failed_report_storage_retains_emergency_evidence() {
    let fixture = Fixture::new("pass");
    fixture.run(None);
    let root = fixture.0.path();
    fs::remove_dir_all(root.join("reports")).unwrap();
    fs::write(root.join("reports"), "storage unavailable").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_review-runner"))
        .arg("run")
        .arg(root.join("config.yaml"))
        .arg(root.join("request.json"))
        .env("REVIEW_CODEX_BIN", root.join("codex"))
        .env("CODEX_HOME", root.join("auth"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("evidence retained"));
    let runtime = fs::read_dir(root.join("runtime"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let id = runtime.file_name().unwrap().to_str().unwrap();
    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(runtime.join(format!("{id}.json"))).unwrap()).unwrap();
    assert_eq!(report["verdict"], "BLOCKED");
    assert!(report["roles"][0]["raw_response"].is_string());
    assert!(!runtime.join("reviewers/first/codex/auth.json").exists());
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
fn cleanup_failure_is_reported_separately_and_preserves_the_report() {
    let fixture = Fixture::new("cleanup");
    let output = fixture.invoke(None);
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let root = fixture.0.path();
    let id = report["run_id"].as_str().unwrap();
    let runtime = root.join("runtime").join(id);
    assert!(report["cleanup_error"].is_string());
    assert!(runtime.exists());
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join(format!("reports/{id}.json"))).unwrap())
            .unwrap();
    assert_eq!(saved["cleanup_error"], report["cleanup_error"]);
    for role in ["first", "second"] {
        let work = runtime.join("reviewers").join(role).join("work");
        fs::set_permissions(work, fs::Permissions::from_mode(0o700)).unwrap();
    }
    fs::remove_dir_all(runtime).unwrap();
}
