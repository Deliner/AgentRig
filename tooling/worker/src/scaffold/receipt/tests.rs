use super::Manifest;
use serde_json::{Value, json};

fn baseline() -> Value {
    json!({
        "manifest_version": 1,
        "package_version": "0.3.0",
        "config_schema": 1,
        "files": {
            "bin/agentrig": {
                "sha256": "recorded-digest",
                "ownership": "runtime",
                "executable": true
            }
        }
    })
}

#[test]
fn old_receipt_without_local_overrides_roundtrips_unchanged() {
    let original = baseline();
    let receipt: Manifest = serde_json::from_value(original.clone()).unwrap();
    assert!(receipt.local.is_empty());
    assert_eq!(serde_json::to_value(receipt).unwrap(), original);
}

#[test]
fn local_deletion_and_replacement_remain_distinct() {
    let mut original = baseline();
    original["local"] = json!({"deleted.md": null, "edited.md": "replacement-digest"});
    let receipt: Manifest = serde_json::from_value(original.clone()).unwrap();
    assert_eq!(receipt.local.get("deleted.md"), Some(&None));
    assert_eq!(receipt.local.get("absent.md"), None);
    assert_eq!(serde_json::to_value(receipt).unwrap(), original);
}

#[test]
fn unknown_receipt_and_entry_fields_are_rejected() {
    let mut receipt = baseline();
    receipt["unexpected"] = json!(true);
    assert!(serde_json::from_value::<Manifest>(receipt).is_err());
    let mut receipt = baseline();
    receipt["files"]["bin/agentrig"]["unexpected"] = json!(true);
    assert!(serde_json::from_value::<Manifest>(receipt).is_err());
}
