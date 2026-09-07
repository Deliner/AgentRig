use super::{digest, read_regular};
use std::fs;

#[test]
fn exact_limit_reads_preserve_bytes_and_fingerprints() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("artifact.bin");
    fs::write(&path, b"abc").unwrap();
    let bytes = read_regular(&path, 3).unwrap();
    assert_eq!(bytes, b"abc");
    assert_eq!(
        digest(&bytes),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert!(read_regular(&path, 2).is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
    fs::write(&path, []).unwrap();
    assert_eq!(read_regular(&path, 0).unwrap(), Vec::<u8>::new());
}

#[test]
fn non_regular_paths_cannot_supply_artifact_bytes() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    let link = root.path().join("link");
    fs::write(&source, b"secret").unwrap();
    std::os::unix::fs::symlink(&source, &link).unwrap();
    assert!(read_regular(&link, 100).is_err());
    assert!(read_regular(root.path(), 100).is_err());
    assert!(read_regular(&root.path().join("missing"), 100).is_err());
    assert_eq!(fs::read(source).unwrap(), b"secret");
}

#[test]
fn json_serialization_failure_preserves_the_previous_artifact() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("result.json");
    let original = serde_json::json!({"status": "accepted"});
    super::json::save(&path, &original).unwrap();
    let before = fs::read(&path).unwrap();
    let invalid = std::collections::BTreeMap::from([(vec![1, 2], "not a JSON object key")]);
    assert!(super::json::save(&path, &invalid).is_err());
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
    let updated = serde_json::json!({"status": "completed"});
    super::json::save(&path, &updated).unwrap();
    let actual: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(actual, updated);
}

#[test]
fn json_replacement_preserves_symlink_targets_and_reports_missing_parents() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("original.json");
    let link = root.path().join("result.json");
    fs::write(&source, b"original").unwrap();
    std::os::unix::fs::symlink(&source, &link).unwrap();
    super::json::save(&link, &serde_json::json!({"new": true})).unwrap();
    assert!(fs::symlink_metadata(&link).unwrap().is_file());
    assert_eq!(fs::read(source).unwrap(), b"original");
    assert!(super::json::save(&root.path().join("missing/result.json"), &0).is_err());
}
