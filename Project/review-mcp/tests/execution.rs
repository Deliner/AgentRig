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

#[test]
fn mcp_lists_configured_tools_and_validates_arguments() {
    use review_runner::mcp::Server;
    use serde_json::json;
    let fixture = Fixture::new("pass");
    let mut server = Server::new(&fixture.0.path().join("config.toml")).unwrap();
    let init = server.message(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}})).unwrap();
    assert_eq!(
        init["result"]["capabilities"]["tools"]["listChanged"],
        false
    );
    let list = server
        .message(json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}))
        .unwrap();
    assert_eq!(list["result"]["tools"][0]["name"], "review_code");
    assert_eq!(
        list["result"]["tools"][0]["inputSchema"]["additionalProperties"],
        false
    );
    let rejected = server.message(json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"review_code","arguments":{"root":".","base":"HEAD","candidate":"HEAD","tool":"override"}}})).unwrap();
    assert_eq!(rejected["error"]["code"], -32602);
    assert!(
        server
            .message(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
            .is_none()
    );
}

#[test]
fn failed_report_storage_retains_emergency_evidence() {
    use std::{fs, process::Command};
    let fixture = Fixture::new("pass");
    fixture.run(None);
    let root = fixture.0.path();
    fs::remove_dir_all(root.join("reports")).unwrap();
    fs::write(root.join("reports"), "storage unavailable").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_review-runner"))
        .arg("run")
        .arg(root.join("config.toml"))
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
