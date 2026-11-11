use super::{Consumer, binaries, inventory::documents};
use crate::shared::testing::{git, worker};
use serde_json::json;
use std::fs;

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

#[test]
fn empty_vcs_directories_require_contracts_without_including_ignored_trees() {
    for binary in binaries() {
        let consumer = versioned();
        fs::create_dir(consumer.root.join("src/docs/drafts")).unwrap();
        consumer.contract("src/docs");
        for nested in ["src/docs/ignored/nested", "src/docs/generated/nested"] {
            fs::create_dir_all(consumer.root.join(nested)).unwrap();
        }
        consumer.configure("version: 1\n", EXCLUDE);
        let before = consumer.snapshot();
        let (code, findings) = consumer.process(&binary);
        assert_eq!(code, Some(1), "{findings}");
        assert_eq!(findings.as_array().unwrap().len(), 1, "{findings}");
        assert_eq!(findings[0]["path"], "src/docs/drafts/architecture.yaml");
        let message = findings[0]["message"].as_str().unwrap();
        assert!(message.contains("missing directory contract"), "{message}");
        assert!(consumer.snapshot() == before, "lint changed the project");
        consumer.contract("src/docs/drafts");
        assert_eq!(consumer.process(&binary), (Some(0), json!([])));
    }
}

#[test]
fn empty_directories_preserve_file_derived_size_measurement() {
    let consumer = versioned();
    fs::create_dir(consumer.root.join("src/docs/drafts")).unwrap();
    consumer.write("lint.yaml", ENTRIES);
    let (code, findings) = consumer.process(worker());
    assert_eq!(code, Some(1), "{findings}");
    let actual: Vec<_> = findings
        .as_array()
        .unwrap()
        .iter()
        .map(|item| &item["actual"])
        .collect();
    assert_eq!(actual, [&json!(1)], "{findings}");
}
