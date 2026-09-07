use review_runner::{
    config, snapshot,
    vcs::{Backend, FileKind, Kind, Repository, external::Adapter},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
    process::Command,
};

fn example() -> Adapter {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/external_vcs.py");
    Adapter {
        command: vec![
            "python3".into(),
            "-B".into(),
            script.to_str().unwrap().into(),
        ],
    }
}

fn hg(root: &Path, args: &[&str]) {
    let output = Command::new("hg")
        .args(args)
        .current_dir(root)
        .env("HGPLAIN", "1")
        .env("HGRCPATH", "")
        .env("HGRCSKIPREPO", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn commit(root: &Path) -> String {
    hg(root, &["addremove"]);
    hg(root, &["commit", "-m", "fixture", "-u", "Test"]);
    Repository::new(root, Kind::Mercurial).resolve(".").unwrap()
}

fn contents(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let kind = entry.file_type().unwrap();
        let directory = kind.is_dir();
        if directory {
            files.extend(contents(&entry.path()));
            continue;
        }
        let symlink = kind.is_symlink();
        let bytes = if symlink {
            fs::read_link(entry.path())
                .unwrap()
                .as_os_str()
                .as_bytes()
                .to_vec()
        } else {
            fs::read(entry.path()).unwrap()
        };
        files.insert(entry.path(), bytes);
    }
    files
}

fn revisions(root: &Path) -> (String, String) {
    hg(root, &["init"]);
    let adapter = example();
    assert_eq!(adapter.head(root).unwrap(), None);
    fs::write(root.join("binary"), [0, 255, 10]).unwrap();
    fs::write(root.join("old name"), "before").unwrap();
    std::os::unix::fs::symlink("old name", root.join("link")).unwrap();
    let base = commit(root);
    fs::rename(root.join("old name"), root.join("new name")).unwrap();
    fs::write(root.join("binary"), [0, 255, 20]).unwrap();
    let candidate = commit(root);
    fs::write(root.join("new name"), "uncommitted").unwrap();
    (base, candidate)
}

#[test]
fn independent_adapter_reads_real_revisions_without_changing_the_repository() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let (base, candidate) = revisions(root);
    let adapter = example();
    let before = contents(root);
    assert_eq!(adapter.head(root).unwrap(), Some(candidate.clone()));
    assert_eq!(adapter.resolve(root, "0").unwrap(), base);
    assert_eq!(
        adapter.parents(root, &candidate).unwrap().as_slice(),
        std::slice::from_ref(&base)
    );
    assert_eq!(adapter.read(root, &base, "binary").unwrap(), [0, 255, 10]);
    assert_eq!(
        adapter.read(root, &candidate, "new name").unwrap(),
        b"before"
    );
    assert_eq!(
        adapter.tree(root, &candidate).unwrap()["link"].kind,
        FileKind::Symlink
    );
    assert_eq!(
        adapter.changed_paths(root, (&base, &candidate)).unwrap(),
        ["binary", "new name", "old name"]
    );
    assert_eq!(
        adapter.working_files(root).unwrap(),
        ["binary", "link", "new name"]
    );
    assert!(
        adapter
            .diff(root, (&base, &candidate))
            .unwrap()
            .contains("new name")
    );
    assert!(adapter.resolve(root, "all()").is_err());
    assert_eq!(contents(root), before);
}

fn reply(value: Value) -> Adapter {
    Adapter {
        command: vec![
            "python3".into(),
            "-c".into(),
            format!("import sys; sys.stdout.write({:?})", value.to_string()),
        ],
    }
}

#[test]
fn configured_external_source_uses_shared_snapshot_visibility_and_file_restrictions() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let (base, candidate) = revisions(root);
    let output = tempfile::tempdir().unwrap();
    let mut scope = config::Repository {
        vcs: Backend::External(example()),
        visible_paths: vec!["binary".into(), "*name".into()],
        contract_paths: vec!["new name".into()],
    };
    let before = contents(root);
    let result = snapshot::prepare(
        root,
        (&base, &candidate),
        &scope,
        &output.path().join("valid"),
    )
    .unwrap();
    assert_eq!(result.candidate, candidate);
    assert_eq!(
        fs::read(output.path().join("valid/new name")).unwrap(),
        b"before"
    );
    scope.visible_paths = vec!["binary".into()];
    assert!(snapshot::check_boundary(root, &base, &candidate, &scope).is_err());
    scope.visible_paths = vec!["**".into()];
    let error = snapshot::prepare(
        root,
        (&base, &candidate),
        &scope,
        &output.path().join("symlink"),
    )
    .unwrap_err();
    assert!(error.to_string().contains("symlink or submodule"));
    assert_eq!(contents(root), before);
}

#[test]
fn backend_selection_preserves_native_yaml_and_rejects_invalid_external_declarations() {
    let native: Backend = config::yaml::decode("mercurial").unwrap();
    assert_eq!(serde_json::to_value(native).unwrap(), json!("mercurial"));
    let invalid: Backend = config::yaml::decode("command: []").unwrap();
    assert!(invalid.validate().is_err());
    assert!(serde_json::from_str::<Backend>(r#"{"command":[],"command":[]}"#).is_err());
    for yaml in [
        "unknown",
        "command: python3",
        "command: [python3]\nextra: true",
    ] {
        assert!(config::yaml::decode::<Backend>(yaml).is_err());
    }
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::External(reply(json!({"version": 1, "result": "revision-42"})));
    assert_eq!(
        backend.source(root.path()).resolve("tip").unwrap(),
        "revision-42"
    );
}

#[test]
fn protocol_rejects_wrong_versions_types_unknown_fields_and_empty_ids() {
    let root = tempfile::tempdir().unwrap();
    for value in [
        json!({"version": 2, "result": "r1"}),
        json!({"version": 1, "result": ["r1"]}),
        json!({"version": 1, "result": "r1", "extra": true}),
        json!({"version": 1, "result": ""}),
        json!({"version": 1, "result": "r1\nr2"}),
    ] {
        assert!(reply(value).resolve(root.path(), "tip").is_err());
    }
    assert_eq!(
        reply(json!({"version": 1, "result": "revision-42"}))
            .resolve(root.path(), "tip")
            .unwrap(),
        "revision-42"
    );
    assert!(
        reply(json!({"version": 1, "result": [256]}))
            .read(root.path(), "r1", "file")
            .is_err()
    );
}

#[test]
fn protocol_rejects_unsafe_paths_duplicate_entries_and_invalid_file_kinds() {
    let root = tempfile::tempdir().unwrap();
    for path in [
        "",
        "../outside",
        "/absolute",
        "a/../b",
        "a/./b",
        "a//b",
        ".git/config",
        ".hg/hgrc",
        "nul\0path",
    ] {
        let entry = json!({"path": path, "kind": "file", "object": "blob-1"});
        assert!(
            reply(json!({"version": 1, "result": [entry]}))
                .tree(root.path(), "r1")
                .is_err()
        );
        assert!(
            reply(json!({"version": 1, "result": [path]}))
                .working_files(root.path())
                .is_err()
        );
    }
    let entry = json!({"path": "file", "kind": "file", "object": "blob-1"});
    assert!(
        reply(json!({"version": 1, "result": [entry, entry]}))
            .tree(root.path(), "r1")
            .is_err()
    );
    let invalid = json!({"path": "file", "kind": "directory", "object": "blob-1"});
    assert!(
        reply(json!({"version": 1, "result": [invalid]}))
            .tree(root.path(), "r1")
            .is_err()
    );
}

#[test]
fn read_adapter_cannot_write_and_unsupported_operations_are_explicit() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("preserve"), "user work").unwrap();
    let adapter = Adapter {
        command: vec![
            "python3".into(),
            "-c".into(),
            "open('preserve','w').write('changed')".into(),
        ],
    };
    assert!(adapter.resolve(root.path(), "tip").is_err());
    assert_eq!(
        fs::read_to_string(root.path().join("preserve")).unwrap(),
        "user work"
    );
    let adapter = Adapter {
        command: vec![
            "python3".into(),
            "-c".into(),
            "import sys; sys.exit(64)".into(),
        ],
    };
    let error = adapter.resolve(root.path(), "tip").unwrap_err().to_string();
    assert!(error.contains("does not support resolve"));
    assert!(Adapter { command: vec![] }.validate().is_err());
}
