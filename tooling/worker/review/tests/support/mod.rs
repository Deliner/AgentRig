use serde_json::{Value, json};
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

pub struct Fixture(pub tempfile::TempDir);
impl Fixture {
    pub fn new(mode: &str) -> Self {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir(root.join("repo")).unwrap();
        git(&root.join("repo"), &["init", "-q"]);
        fs::create_dir(root.join("repo/src")).unwrap();
        fs::write(root.join("repo/src/value.py"), "value = 1\n").unwrap();
        git(&root.join("repo"), &["add", "."]);
        git(
            &root.join("repo"),
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-qm",
                "base",
            ],
        );
        fs::create_dir(root.join("auth")).unwrap();
        fs::write(root.join("auth/auth.json"), "{}").unwrap();
        fs::write(root.join("auth/config.toml"), "INVALID HOST CONFIG").unwrap();
        fs::write(root.join("codex-code-mode-host"), "unused").unwrap();
        let script =
            include_str!("critic.py").replace(r#"MODE = "pass""#, &format!("MODE = {mode:?}"));
        fs::write(root.join("codex"), script).unwrap();
        fs::set_permissions(root.join("codex"), fs::Permissions::from_mode(0o755)).unwrap();
        resources(root);
        let fixture = Self(directory);
        fixture.mode(mode);
        fixture
    }
    pub fn mode(&self, mode: &str) {
        let script =
            include_str!("critic.py").replace(r#"MODE = "pass""#, &format!("MODE = {mode:?}"));
        fs::write(self.0.path().join("codex"), script).unwrap();
    }
    pub fn invoke(&self, previous: Option<&Path>) -> std::process::Output {
        let root = self.0.path();
        let request = json!({"root":root.join("repo"),"base":git(&root.join("repo"), &["rev-list", "--max-parents=0", "HEAD"]),"candidate":"HEAD","tool":"review_code","previous_report":previous});
        fs::write(
            root.join("request.json"),
            serde_json::to_vec(&request).unwrap(),
        )
        .unwrap();
        Command::new(env!("CARGO_BIN_EXE_review-runner"))
            .args(["run"])
            .arg(root.join("config.yaml"))
            .arg(root.join("request.json"))
            .env("REVIEW_CODEX_BIN", root.join("codex"))
            .env("CODEX_HOME", root.join("auth"))
            .output()
            .unwrap()
    }
    pub fn run(&self, previous: Option<&Path>) -> Value {
        let root = self.0.path();
        let output = self.invoke(previous);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(
            !root
                .join("runtime")
                .join(report["run_id"].as_str().unwrap())
                .exists()
        );
        assert!(report["cleanup_error"].is_null());
        report
    }
}
pub fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
fn resources(root: &Path) {
    fs::write(
        root.join("config.yaml"),
        r#"schema_version: 1
runner:
  runtime_root: runtime
  report_root: reports
  isolation: bubblewrap
  parallelism: 2
  timeout_seconds: 3
  format_attempts: 2
reviewers:
  first:
    model: test
    reasoning_effort: high
    prompt: prompt.md
  second:
    model: test
    reasoning_effort: high
    prompt: prompt.md
tools:
  review_code:
    description: Test review
    reviewers:
    - first
    - second
    project_config: project.yaml
"#,
    )
    .unwrap();
    fs::write(root.join("project.yaml"), "schema_version: 1\nrepository:\n  visible_paths:\n  - src/**\n  contract_paths: []\nreview:\n  contract: contract.json\n").unwrap();
    fs::write(root.join("prompt.md"), "Check C-1.").unwrap();
    fs::write(root.join("contract.json"), serde_json::to_vec(&json!({"schema_version":1,"requirements":[{"id":"C-1","text":"Value is one","reviewers":["first","second"],"allow_na":false}]})).unwrap()).unwrap();
}
