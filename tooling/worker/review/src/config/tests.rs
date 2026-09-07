use review_runner::config;
use std::{fs, path::Path};

fn fixture(root: &Path) -> std::path::PathBuf {
    fs::write(root.join("prompt.md"), "Review assigned requirements.").unwrap();
    fs::write(root.join("contract.json"), r#"{"schema_version":1,"requirements":[{"id":"C-1","text":"Correctness","reviewers":["critic"],"allow_na":false}]}"#).unwrap();
    fs::write(root.join("project.yaml"), "schema_version: 1\nrepository:\n  visible_paths:\n  - src/**\n  contract_paths: []\nreview:\n  contract: contract.json\n").unwrap();
    let path = root.join("runner.yaml");
    fs::write(&path, "schema_version: 1\nrunner:\n  runtime_root: runtime\n  report_root: reports\n  isolation: bubblewrap\n  parallelism: 3\n  timeout_seconds: 900\n  format_attempts: 3\nreviewers:\n  critic:\n    model: gpt-5.6-luna\n    reasoning_effort: high\n    prompt: prompt.md\ntools:\n  review_code:\n    description: Review code\n    reviewers:\n    - critic\n    project_config: project.yaml\n").unwrap();
    path
}
#[test]
fn resolves_resources_relative_to_config() {
    let root = tempfile::tempdir().unwrap();
    let path = fixture(root.path());
    let result = config::load(&path).unwrap();
    assert_eq!(
        result.reviewers["critic"].prompt,
        root.path().join("prompt.md")
    );
    assert_eq!(
        result.tools["review_code"].project_config,
        root.path().join("project.yaml")
    );
    assert_eq!(result.runner.runtime_root, root.path().join("runtime"));
}
#[test]
fn rejects_invalid_configuration_before_execution() {
    let root = tempfile::tempdir().unwrap();
    let path = fixture(root.path());
    let source = fs::read_to_string(&path).unwrap();
    for (old, new) in [
        ("parallelism: 3", "parallelism: 0"),
        ("schema_version: 1", "schema_version: 2"),
        ("- critic", "- unknown"),
        ("- critic", "- critic\n    - critic"),
        ("isolation: bubblewrap", "isolation: none"),
        ("parallelism: 3", "parallelism: '3'"),
        ("schema_version: 1", "schema_version: 1\nunknown: true"),
    ] {
        assert!(source.contains(old));
        fs::write(&path, source.replace(old, new)).unwrap();
        assert!(config::load(&path).is_err(), "accepted {new}");
    }
}

#[test]
fn review_loading_rejects_legacy_toml_without_fallback() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("review.toml");
    fs::write(&path, "schema_version=1\n[runner]\nparallelism=3\n").unwrap();
    let error = format!("{:#}", config::load(&path).unwrap_err());
    assert!(error.contains("YAML configuration") && error.contains("review.toml"));
}

#[test]
fn frontend_is_explicit_and_unsupported_executors_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let path = fixture(root.path());
    let source = fs::read_to_string(&path).unwrap();
    assert_eq!(
        config::load(&path).unwrap().reviewers["critic"].frontend,
        "codex"
    );
    let explicit = source.replace("  critic:", "  critic:\n    frontend: codex");
    fs::write(&path, &explicit).unwrap();
    let loaded = config::load(&path).unwrap();
    assert_eq!(loaded.reviewers["critic"].frontend, "codex");
    let recorded = serde_json::to_value(&loaded).unwrap();
    assert_eq!(recorded["reviewers"]["critic"]["frontend"], "codex");
    fs::write(
        &path,
        explicit.replace("frontend: codex", "frontend: unknown"),
    )
    .unwrap();
    let error = config::load(&path).unwrap_err().to_string();
    assert!(error.contains("unsupported frontend"), "{error}");
    assert!(
        error.contains("critic") && error.contains("supported: codex"),
        "{error}"
    );
}
