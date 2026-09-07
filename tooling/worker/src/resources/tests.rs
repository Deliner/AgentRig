use super::{Bundle, digest};
use std::{fs, os::unix::fs::PermissionsExt};

#[test]
fn read_and_verify_reject_changed_content_and_executable_mode() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("input");
    fs::write(&path, b"original").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    let mut bundle = Bundle::new("service");
    assert_eq!(bundle.read(&path).unwrap(), b"original");
    bundle.verify().unwrap();
    fs::write(&path, b"changed").unwrap();
    assert!(bundle.read(&path).is_err());
    assert!(bundle.verify().is_err());
    fs::write(&path, b"original").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(bundle.read(&path).is_err());
    assert!(bundle.verify().is_err());
}

#[test]
fn destinations_preserve_identity_and_reject_conflicting_content_or_mode() {
    let mut bundle = Bundle::new("service");
    let first = bundle.put("run", b"abc".to_vec(), false).unwrap();
    assert_eq!(first, bundle.put("run", b"abc".to_vec(), false).unwrap());
    assert_ne!(first, bundle.put("run", b"abc".to_vec(), true).unwrap());
    assert!(bundle.put("../run", Vec::new(), false).is_err());
    assert_eq!(
        digest(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let root = tempfile::tempdir().unwrap();
    for (name, bytes, mode) in [
        ("a", b"abc", 0o644),
        ("b", b"def", 0o644),
        ("c", b"abc", 0o755),
    ] {
        let path = root.path().join(name);
        fs::write(&path, bytes).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
    }
    bundle.copy_at(&root.path().join("a"), "fixed").unwrap();
    bundle.copy_at(&root.path().join("a"), "fixed").unwrap();
    assert!(bundle.copy_at(&root.path().join("b"), "fixed").is_err());
    assert!(bundle.copy_at(&root.path().join("c"), "fixed").is_err());
    assert_eq!(bundle.files["fixed"].bytes, b"abc");
}

#[test]
fn directory_identity_uses_relative_contents_and_rejects_symlinks() {
    let root = tempfile::tempdir().unwrap();
    let left = root.path().join("left/assets");
    let right = root.path().join("right/assets");
    fs::create_dir_all(&left).unwrap();
    fs::create_dir_all(&right).unwrap();
    for (directory, names) in [(&left, ["a", "b"]), (&right, ["b", "a"])] {
        for name in names {
            let path = directory.join(name);
            fs::write(&path, name).unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o644)).unwrap();
        }
    }
    let mut bundle = Bundle::new("service");
    let destination = bundle.directory(&left).unwrap();
    assert_eq!(destination, bundle.directory(&right).unwrap());
    assert_eq!(bundle.files.len(), 2);
    assert_eq!(bundle.files[&format!("{destination}/a")].bytes, b"a");
    std::os::unix::fs::symlink(left.join("a"), right.join("link")).unwrap();
    assert!(bundle.directory(&right).is_err());
    bundle.verify().unwrap();
}
