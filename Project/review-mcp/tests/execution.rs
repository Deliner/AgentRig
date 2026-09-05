mod support;
use support::Fixture;

#[test]
fn isolated_parallel_reviews_persist_exact_answers_and_cleanup() {
    let fixture = Fixture::new("pass");
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
    let id = report["run_id"].as_str().unwrap();
    assert!(fixture.0.path().join(format!("reports/{id}.md")).is_file());
    assert_ne!(fixture.run(None)["run_id"], id);
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
