use review_runner::{
    contract::{Contract, Requirement},
    response::{self, Expected},
};
use serde_json::{Value, json};

fn expected() -> Expected {
    Expected {
        run_id: "run-1".into(),
        candidate: "a".repeat(40),
        contract_digest: "b".repeat(64),
        role: "correctness".into(),
        contract: Contract {
            schema_version: 1,
            requirements: vec![Requirement {
                id: "C-1".into(),
                text: "Preserve behavior".into(),
                reviewers: vec!["correctness".into()],
                allow_na: false,
            }],
        },
    }
}
fn answer() -> Value {
    json!({"run_id":"run-1", "candidate":"a".repeat(40), "contract_digest":"b".repeat(64),
        "role":"correctness", "verdict":"PASS", "checks":[{"contract_id":"C-1", "status":"PASS",
            "evidence":"src/lib.rs:1", "finding":"", "minimal_fix":""}], "observations":[]})
}
fn rejects(value: Value) {
    assert!(response::validate(&serde_json::to_vec(&value).unwrap(), &expected()).is_err());
}
#[test]
fn validates_identity_coverage_and_schema() {
    assert!(response::validate(&serde_json::to_vec(&answer()).unwrap(), &expected()).is_ok());
    for key in ["run_id", "candidate", "contract_digest", "role", "verdict"] {
        let mut changed = answer();
        changed[key] = json!("other");
        rejects(changed);
    }
    let mut missing = answer();
    missing["checks"] = json!([]);
    rejects(missing);
    let mut duplicate = answer();
    duplicate["checks"]
        .as_array_mut()
        .unwrap()
        .push(answer()["checks"][0].clone());
    rejects(duplicate);
    let mut unknown = answer();
    unknown["checks"][0]["contract_id"] = json!("unknown");
    rejects(unknown);
    let mut extra = answer();
    extra["unexpected"] = json!(true);
    rejects(extra);
}
#[test]
fn validates_failure_and_na_semantics() {
    for status in ["FAIL", "BLOCKED", "N/A"] {
        let mut changed = answer();
        changed["checks"][0]["status"] = json!(status);
        rejects(changed);
    }
    let mut failed = answer();
    failed["verdict"] = json!("FAIL");
    failed["checks"][0]["status"] = json!("FAIL");
    rejects(failed.clone());
    failed["checks"][0]["finding"] = json!("Incorrect result");
    failed["checks"][0]["minimal_fix"] = json!("Return the expected value");
    assert!(response::validate(&serde_json::to_vec(&failed).unwrap(), &expected()).is_ok());
    failed["checks"][0]["late_finding"] = json!(true);
    rejects(failed);
}
#[test]
fn rejects_missing_oversized_and_symlink_outputs() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("review.json");
    assert!(response::file(&file, &expected()).is_err());
    let original = root.path().join("answer.json");
    std::fs::write(&original, serde_json::to_vec(&answer()).unwrap()).unwrap();
    std::os::unix::fs::symlink(&original, &file).unwrap();
    assert!(response::file(&file, &expected()).is_err());
    std::fs::remove_file(&file).unwrap();
    std::fs::write(&file, vec![b' '; response::MAX_BYTES as usize + 1]).unwrap();
    assert!(response::file(&file, &expected()).is_err());
}
#[test]
fn rejects_incomplete_or_unknown_assignments() {
    let expected = expected();
    assert!(expected.contract.validate(&["correctness".into()]).is_ok());
    assert!(expected.contract.validate(&["other".into()]).is_err());
    let mut contract = expected.contract;
    contract.requirements.push(contract.requirements[0].clone());
    assert!(contract.validate(&["correctness".into()]).is_err());
}
