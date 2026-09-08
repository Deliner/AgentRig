#[path = "../../../testing/mod.rs"]
mod support;
use serde_json::{Value, json};
use std::fs;
use support::Fixture;

fn claude(mode: &str) -> Fixture {
    let fixture = Fixture::new(mode);
    let path = fixture.0.path().join("config.yaml");
    let mut config: Value = review_runner::config::yaml::read(&path).unwrap();
    config["reviewers"]["second"]["frontend"] = json!("claude-code");
    config["reviewers"]["second"]["credentials"] = json!({"env":{
        "CLAUDE_CODE_OAUTH_TOKEN":"REVIEW_TEST_CLAUDE_TOKEN"}});
    fs::write(path, review_runner::config::yaml::encode(&config).unwrap()).unwrap();
    fixture
}

#[test]
fn mixed_review_clients_share_validation_and_preserve_the_source() {
    let fixture = claude("correction");
    let report = fixture.run(None);
    assert_eq!(report["verdict"], "PASS", "{report:#}");
    assert_eq!(report["roles"][0]["frontend"], "codex");
    assert_eq!(report["roles"][1]["frontend"], "claude-code");
    for role in report["roles"].as_array().unwrap() {
        assert_eq!(role["format_attempts"], 2);
        assert!(role["response"].is_object());
    }
    assert_eq!(
        fs::read_to_string(fixture.0.path().join("repo/src/value.py")).unwrap(),
        "value = 1\n"
    );
    assert!(!report.to_string().contains("fixture-token"));
}

#[test]
fn claude_exhaustion_and_timeout_remain_technical_failures() {
    for mode in ["exhausted", "timeout"] {
        let fixture = claude(mode);
        let report = fixture.run(None);
        assert_eq!(report["verdict"], "BLOCKED", "{report:#}");
        let role = &report["roles"][1];
        assert_eq!(role["frontend"], "claude-code");
        let error = role["technical_error"].as_str().unwrap();
        let timed_out = mode == "timeout";
        let expected = if timed_out {
            "timeout"
        } else {
            "format attempt limit"
        };
        assert!(error.contains(expected), "{report:#}");
    }
}

#[test]
fn claude_configuration_rejects_unsupported_auth_effort_and_sandbox_overrides() {
    let fixture = claude("pass");
    let path = fixture.0.path().join("config.yaml");
    let baseline: Value = review_runner::config::yaml::read(&path).unwrap();
    for (key, value, message) in [
        (
            "reasoning_effort",
            json!("minimal"),
            "unsupported claude-code reasoning effort",
        ),
        ("credentials", json!({"env":{}}), "environment reference"),
        (
            "credentials",
            json!({"codex_auth_file_env":"AUTH_FILE"}),
            "does not use",
        ),
        (
            "credentials",
            json!({"env":{"CLAUDE_CONFIG_DIR":"OVERRIDE"}}),
            "owned by the sandbox",
        ),
        (
            "credentials",
            json!({"env":{"CLAUDE_CODE_OAUTH_TOKEN":"not a variable"}}),
            "variable names",
        ),
    ] {
        let mut config = baseline.clone();
        config["reviewers"]["second"][key] = value;
        fs::write(&path, review_runner::config::yaml::encode(&config).unwrap()).unwrap();
        let error = review_runner::config::load(&path).unwrap_err().to_string();
        assert!(error.contains(message), "{error}");
    }
}

#[test]
fn missing_claude_reference_is_a_role_error_without_exposing_host_auth() {
    let fixture = claude("pass");
    let path = fixture.0.path().join("config.yaml");
    let source = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        source.replace("REVIEW_TEST_CLAUDE_TOKEN", "REVIEW_TEST_MISSING_TOKEN"),
    )
    .unwrap();
    let report = fixture.run(None);
    assert_eq!(report["verdict"], "BLOCKED", "{report:#}");
    assert!(
        report["roles"][1]["technical_error"]
            .as_str()
            .unwrap()
            .contains("missing credential reference REVIEW_TEST_MISSING_TOKEN")
    );
    assert_eq!(report["roles"][1]["format_attempts"], 0);
}

#[test]
fn codex_explicit_api_reference_does_not_require_the_host_auth_file() {
    let fixture = Fixture::new("pass");
    let root = fixture.0.path();
    fs::remove_dir_all(root.join("auth")).unwrap();
    let path = root.join("config.yaml");
    let mut config: Value = review_runner::config::yaml::read(&path).unwrap();
    for role in ["first", "second"] {
        config["reviewers"][role]["credentials"] = json!({"env":{
            "OPENAI_API_KEY":"REVIEW_TEST_CLAUDE_TOKEN"}});
    }
    fs::write(path, review_runner::config::yaml::encode(&config).unwrap()).unwrap();
    let report = fixture.run(None);
    assert_eq!(report["verdict"], "PASS", "{report:#}");
    assert!(!report.to_string().contains("fixture-token"));
}
