// DECISION: D106
use super::{Error, OpenOptions, Permissions, PermissionsExt};
use std::{io::ErrorKind, path::Path};

#[test]
fn errors_name_the_action_and_path_and_keep_the_kind() {
    let directory = tempfile::tempdir().unwrap();
    let missing = directory.path().join("missing");
    let cases: Vec<(&str, Error)> = vec![
        ("read", super::read(&missing).unwrap_err()),
        ("read", super::read_to_string(&missing).unwrap_err()),
        ("open", super::open(&missing).unwrap_err()),
        ("inspect", super::metadata(&missing).unwrap_err()),
        ("inspect", super::symlink_metadata(&missing).unwrap_err()),
        ("read link", super::read_link(&missing).unwrap_err()),
        ("read directory", super::read_dir(&missing).err().unwrap()),
        ("remove", super::remove_file(&missing).unwrap_err()),
        ("remove", super::remove_dir_all(&missing).unwrap_err()),
        (
            "set permissions on",
            super::set_permissions(&missing, Permissions::from_mode(0o644)).unwrap_err(),
        ),
    ];
    for (action, error) in cases {
        assert_eq!(error.to_string(), format!("{action} {}", missing.display()));
        assert_eq!(error.kind(), ErrorKind::NotFound);
        assert!(std::error::Error::source(&error).is_some(), "{action}");
    }
    let link = missing.join("link");
    let error = super::symlink("target", &link).unwrap_err();
    assert_eq!(error.to_string(), format!("link {}", link.display()));
    let error = super::write(&link, "text").unwrap_err();
    assert_eq!(error.to_string(), format!("write {}", link.display()));
}

#[test]
fn wrappers_perform_the_operations_they_name() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path().join("a/b");
    super::create_dir_all(&root).unwrap();
    let file = root.join("file");
    super::write(&file, "text").unwrap();
    assert_eq!(super::read_to_string(&file).unwrap(), "text");
    assert_eq!(super::read(&file).unwrap(), b"text");
    super::set_permissions(&file, Permissions::from_mode(0o755)).unwrap();
    let mode = super::metadata(&file).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o755);
    let link = root.join("link");
    super::symlink("file", &link).unwrap();
    assert!(
        super::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(super::read_link(&link).unwrap(), Path::new("file"));
    let exclusive = OpenOptions::new().write(true).create_new(true).clone();
    let error = super::open_with(&exclusive, &file).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::AlreadyExists);
    assert_eq!(error.to_string(), format!("open {}", file.display()));
    let mut names: Vec<String> = super::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, ["file", "link"]);
    super::remove_file(&link).unwrap();
    super::remove_dir_all(directory.path().join("a")).unwrap();
    assert_eq!(
        super::read_dir(&root).err().unwrap().kind(),
        ErrorKind::NotFound
    );
}
