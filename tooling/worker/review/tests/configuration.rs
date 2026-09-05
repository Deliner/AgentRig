use review_runner::config;
use std::{fs, path::Path};

fn fixture(root: &Path) -> std::path::PathBuf {
    fs::write(root.join("prompt.md"), "Review assigned requirements.").unwrap();
    fs::write(root.join("contract.json"), r#"{"schema_version":1,"requirements":[{"id":"C-1","text":"Correctness","reviewers":["critic"],"allow_na":false}]}"#).unwrap();
    fs::write(root.join("project.toml"), "schema_version=1\n[repository]\nvisible_paths=['src/**']\ncontract_paths=[]\n[review]\ncontract='contract.json'\n").unwrap();
    let path = root.join("runner.toml");
    fs::write(&path, "schema_version=1\n[runner]\nruntime_root='runtime'\nreport_root='reports'\nisolation='bubblewrap'\nparallelism=3\ntimeout_seconds=900\nformat_attempts=3\n[reviewers.critic]\nmodel='gpt-5.6-luna'\nreasoning_effort='high'\nprompt='prompt.md'\n[tools.review_code]\ndescription='Review code'\nreviewers=['critic']\nproject_config='project.toml'\n").unwrap();
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
        root.path().join("project.toml")
    );
    assert_eq!(result.runner.runtime_root, root.path().join("runtime"));
}
#[test]
fn rejects_invalid_configuration_before_execution() {
    let root = tempfile::tempdir().unwrap();
    let path = fixture(root.path());
    let source = fs::read_to_string(&path).unwrap();
    for (old, new) in [
        ("parallelism=3", "parallelism=0"),
        ("schema_version=1", "schema_version=2"),
        ("reviewers=['critic']", "reviewers=['unknown']"),
        ("reviewers=['critic']", "reviewers=['critic','critic']"),
        ("isolation='bubblewrap'", "isolation='none'"),
    ] {
        fs::write(&path, source.replace(old, new)).unwrap();
        assert!(config::load(&path).is_err(), "accepted {new}");
    }
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
    let explicit = source.replace("[reviewers.critic]", "[reviewers.critic]\nfrontend='codex'");
    fs::write(&path, &explicit).unwrap();
    let loaded = config::load(&path).unwrap();
    assert_eq!(loaded.reviewers["critic"].frontend, "codex");
    let recorded = serde_json::to_value(&loaded).unwrap();
    assert_eq!(recorded["reviewers"]["critic"]["frontend"], "codex");
    fs::write(
        &path,
        explicit.replace("frontend='codex'", "frontend='unknown'"),
    )
    .unwrap();
    let error = config::load(&path).unwrap_err().to_string();
    assert!(error.contains("unsupported frontend"), "{error}");
    assert!(
        error.contains("critic") && error.contains("supported: codex"),
        "{error}"
    );
}
