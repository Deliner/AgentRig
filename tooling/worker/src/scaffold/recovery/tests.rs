use super::{active, directory, guard, journal, load, receipt};
use serde_json::{Value, json};
use std::{fs, path::Path};

fn save(root: &Path, project: &Path) {
    let operation = directory(root).unwrap().join("operation");
    fs::create_dir_all(&operation).unwrap();
    let plan = serde_json::to_vec(&json!({
        "version": 1, "project": project, "from_version": "0.2.0",
        "to_version": "0.3.0", "baseline": "baseline", "files": {}, "checks": [],
        "manifest": {"manifest_version": 1, "package_version": "0.3.0",
            "config_schema": 1, "files": {}}
    }))
    .unwrap();
    fs::write(operation.join("plan.json"), &plan).unwrap();
    fs::write(
        operation.join("journal.json"),
        serde_json::to_vec(&json!({
            "version": 1, "plan": "update-plan.json", "plan_sha256": receipt::checksum(&plan),
            "phase": "applying", "completed": [], "restored": [], "checks": {},
            "next_action": "resume update"
        }))
        .unwrap(),
    )
    .unwrap();
}

fn fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join("worker.toml"),
        "[paths]\nruntime = '.runtime'\n",
    )
    .unwrap();
    save(root.path(), root.path());
    root
}

#[test]
fn corrupt_plan_and_foreign_project_are_not_recovery_state() {
    let root = fixture();
    let operation = directory(root.path()).unwrap().join("operation");
    fs::write(operation.join("plan.json"), "tampered").unwrap();
    assert_eq!(
        journal(root.path()).err().unwrap().to_string(),
        "saved upgrade plan changed"
    );
    save(root.path(), &root.path().join("another-project"));
    assert_eq!(
        load(root.path()).err().unwrap().to_string(),
        "operation belongs to another project"
    );
}

#[test]
fn journal_version_and_finished_phases_retain_their_meaning() {
    let root = fixture();
    assert!(active(root.path()).unwrap());
    assert!(guard(root.path()).is_err());
    let path = directory(root.path())
        .unwrap()
        .join("operation/journal.json");
    let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    for phase in ["applied", "rolled-back"] {
        value["phase"] = json!(phase);
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(!active(root.path()).unwrap());
        assert!(guard(root.path()).is_ok());
    }
    value["version"] = json!(2);
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert_eq!(
        journal(root.path()).err().unwrap().to_string(),
        "unsupported upgrade journal"
    );
}
