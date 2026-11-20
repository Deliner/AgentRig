// DECISION: D106
use super::{Error, OpenOptions, Permissions, PermissionsExt};
use std::{ffi::OsStr, io::ErrorKind, path::Path};

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
        ("resolve", super::canonicalize(&missing).unwrap_err()),
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
    let error = super::symlink(OsStr::new("target"), &link).unwrap_err();
    assert_eq!(error.to_string(), format!("link {}", link.display()));
    let error = super::write(&link, "text").unwrap_err();
    assert_eq!(error.to_string(), format!("write {}", link.display()));
}

// DECISION: D125
#[test]
fn existence_and_handle_metadata_name_the_path() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("file");
    super::write(&file, "text").unwrap();
    assert!(super::try_exists(&file).unwrap());
    assert!(!super::try_exists(directory.path().join("missing")).unwrap());
    let beneath = file.join("child");
    let error = super::try_exists(&beneath).unwrap_err();
    assert_eq!(error.to_string(), format!("inspect {}", beneath.display()));
    let handle = super::open(&file).unwrap();
    assert_eq!(super::file_metadata(&handle, &file).unwrap().len(), 4);
}

#[test]
fn only_a_missing_path_or_a_non_directory_is_absent() {
    let directory = tempfile::tempdir().unwrap();
    let file = directory.path().join("file");
    super::write(&file, "text").unwrap();
    let read = super::present(super::read(&file)).unwrap();
    assert_eq!(read.as_deref(), Some(&b"text"[..]));
    let missing = directory.path().join("missing");
    assert!(super::present(super::read(&missing)).unwrap().is_none());
    assert!(super::present(super::read_dir(&file)).unwrap().is_none());
    let looped = directory.path().join("loop");
    super::symlink(OsStr::new("loop"), &looped).unwrap();
    let error = super::present(super::read_dir(&looped)).err().unwrap();
    assert_eq!(
        error.to_string(),
        format!("read directory {}", looped.display())
    );
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
    super::symlink(OsStr::new("file"), &link).unwrap();
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
