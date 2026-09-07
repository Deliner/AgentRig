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
