use super::{Consumer, inventory::documents};
use crate::shared::testing::{Run, git, launch, run, worker};
use std::{fs, path::Path};

const EXCLUDE: &str = "exclude: ['src/docs/generated', 'src/docs/generated/**']\nversion: 1\n";
const ENTRIES: &str = r#"{"version": 1, "config_skill": ".agents/skills/repair/SKILL.md", "rules": [
  {"id": "size", "kind": "directory-entries", "target": "directory", "include": ["src/docs"],
   "error": 0, "warning_skill": ".agents/skills/repair/SKILL.md",
   "error_skill": ".agents/skills/repair/SKILL.md"}]}"#;

fn versioned() -> Consumer {
    let consumer = documents();
    git(&consumer.root, &["init", "-q"]);
    consumer.write(".gitignore", "ignored/\n");
    consumer
}

fn lint(binary: &Path, consumer: &Consumer) -> (Run, serde_json::Value) {
    let result = run(&mut launch(binary, &consumer.root, &["lint", "--json"]), "");
    let findings = serde_json::from_str(&result.stdout).unwrap();
    (result, findings)
}

#[test]
fn empty_vcs_directories_require_contracts_without_including_ignored_trees() {
    for binary in [
        worker().to_owned(),
        worker().with_file_name("agentrig-lint"),
    ] {
        let consumer = versioned();
        fs::create_dir(consumer.root.join("src/docs/drafts")).unwrap();
        consumer.contract("src/docs");
        for nested in ["src/docs/ignored/nested", "src/docs/generated/nested"] {
            fs::create_dir_all(consumer.root.join(nested)).unwrap();
        }
        consumer.configure("version: 1\n", EXCLUDE);
        let before = consumer.snapshot();
        let (result, findings) = lint(&binary, &consumer);
        assert_eq!(result.code, Some(1), "{}", result.output());
        assert_eq!(findings[0]["path"], "src/docs/drafts/architecture.yaml");
        assert_eq!(findings.as_array().unwrap().len(), 1, "{findings}");
        let message = findings[0]["message"].as_str().unwrap();
        assert!(message.contains("missing directory contract"), "{message}");
        assert!(consumer.snapshot() == before, "lint changed the project");
        consumer.contract("src/docs/drafts");
        let (result, findings) = lint(&binary, &consumer);
        assert_eq!(result.code, Some(0), "{}", result.output());
        assert_eq!(findings, serde_json::json!([]));
    }
}

#[test]
fn empty_directories_preserve_file_derived_size_measurement() {
    let consumer = versioned();
    fs::create_dir(consumer.root.join("src/docs/drafts")).unwrap();
    consumer.write("lint.yaml", ENTRIES);
    let (result, findings) = lint(worker(), &consumer);
    assert_eq!(result.code, Some(1), "{}", result.output());
    let actual: Vec<_> = findings
        .as_array()
        .unwrap()
        .iter()
        .map(|item| &item["actual"])
        .collect();
    assert_eq!(actual, [&serde_json::json!(1)], "{findings}");
}
