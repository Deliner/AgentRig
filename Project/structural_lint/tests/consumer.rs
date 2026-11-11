use super::architecture::FRONTMATTER;
use crate::shared::testing::{Run, launch, run, worker};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const REPAIR: &str = ".agents/skills/repair/SKILL.md";
pub const SKILL: &str = ".agents/skills/refactor-large-file/SKILL.md";
pub const EXTENSIONS: &str = r#"extensions: [".rs", ".py", ".ts"]"#;
pub const CONFIG: &str = r#"version: 1
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

pub fn prepare(root: &Path, config: &str) {
    for skill in [SKILL, REPAIR] {
        write(root, skill, FRONTMATTER);
    }
    write(root, "lint.yaml", config);
    fs::create_dir_all(root.join("src")).unwrap();
}

pub fn consumer(config: &str) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    prepare(&root, config);
    (temp, root)
}

pub fn write(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

pub fn command(root: &Path, name: &str, flags: &[&str]) -> Run {
    let mut args = vec![name, "--config", "lint.yaml"];
    args.extend(flags);
    run(&mut launch(worker(), root, &args), "")
}

pub fn parsed(result: &Run) -> Vec<Value> {
    let findings = serde_json::from_str(&result.stdout);
    findings.unwrap_or_else(|error| panic!("{error}: {}", result.output()))
}

pub fn lint(root: &Path) -> (Option<i32>, Vec<Value>) {
    let result = command(root, "lint", &["--json", "--warnings"]);
    (result.code, parsed(&result))
}

pub fn check(root: &Path) -> (Option<i32>, Vec<Value>) {
    let result = command(root, "lint-config-check", &["--json"]);
    (result.code, parsed(&result))
}
