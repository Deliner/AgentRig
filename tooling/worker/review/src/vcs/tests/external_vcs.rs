pub mod external_fixture;
use external_fixture::{commit, contents, example, revisions};
use review_runner::vcs::{Backend, FileKind, Kind, external::Adapter};
use serde_json::{Value, json};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

#[test]
fn configured_initialization_preserves_files_and_repeated_repository_state() {
    for backend in [
        Kind::Git.into(),
        Kind::Mercurial.into(),
        Backend::External(example()),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::write(root.join("consumer.txt"), "untracked consumer input").unwrap();
        backend.initialize(root, "trunk").unwrap();
        let observed = backend.source(root).observe().unwrap();
        assert_eq!(observed.branch, "trunk");
        assert!(observed.revision.is_empty());
        assert!(observed.status.contains("consumer.txt"));
        let before = contents(root);
        backend.initialize(root, "another-base").unwrap();
        assert_eq!(backend.source(root).observe().unwrap().branch, "trunk");
        assert_eq!(contents(root), before);
    }
}

#[test]
fn private_initialization_preserves_existing_committed_and_mismatched_repositories() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let (_, candidate) = revisions(root);
    let backend = Backend::External(example());
    let before = contents(root);
    backend.initialize(root, "another-base").unwrap();
    assert_eq!(backend.source(root).head().unwrap(), Some(candidate));
    assert_eq!(contents(root), before);
    let git = tempfile::tempdir().unwrap();
    Backend::Native(Kind::Git)
        .initialize(git.path(), "main")
        .unwrap();
    let before = contents(git.path());
    assert!(backend.initialize(git.path(), "trunk").is_err());
    assert_eq!(contents(git.path()), before);
    assert!(!git.path().join(".hg").exists());
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
fn private_commit_context_requires_a_branch_and_boolean_merge_state() {
    let root = tempfile::tempdir().unwrap();
    for invalid in [
        json!(["task/example", "false"]),
        json!([42, false]),
        json!(["task/example", false, true]),
    ] {
        let backend = Backend::External(reply(json!({"version":1,"result":invalid})));
        assert!(
            backend
                .source(root.path())
                .commit_context(Some("revision-42"))
                .is_err()
        );
    }
    let backend = Backend::External(reply(json!({"version":1,"result":["task/example",false]})));
    assert_eq!(
        backend
            .source(root.path())
            .commit_context(Some("revision-42"))
            .unwrap(),
        ("task/example".into(), false)
    );
}

#[test]
fn external_revision_export_preserves_committed_inputs_and_original_checkout() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let (_, candidate) = revisions(root);
    fs::write(root.join("run"), b"#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(root.join("run"), fs::Permissions::from_mode(0o755)).unwrap();
    let executable = commit(root);
    fs::write(root.join("run"), "dirty").unwrap();
    let before = contents(root);
    let backend = Backend::External(example());
    let source = backend.source(root);
    let exported = source.export_revision(&candidate).unwrap();
    assert_eq!(
        fs::read(exported.path().join("new name")).unwrap(),
        b"before"
    );
    assert_eq!(
        fs::read(exported.path().join("binary")).unwrap(),
        [0, 255, 20]
    );
    assert_eq!(
        fs::read_link(exported.path().join("link")).unwrap(),
        Path::new("old name")
    );
    assert!(!exported.path().join("old name").exists());
    assert!(!exported.path().join("run").exists());
    assert!(!exported.path().join(".hg").exists());
    let exported = source.export_revision(&executable).unwrap();
    let script = exported.path().join("run");
    assert_eq!(fs::read(&script).unwrap(), b"#!/bin/sh\nexit 0\n");
    assert_eq!(
        fs::metadata(script).unwrap().permissions().mode() & 0o777,
        0o755
    );
    assert_eq!(contents(root), before);
}

fn export_adapter(entries: Value) -> Backend {
    let responses =
        json!({"resolve": "revision-42", "tree": entries, "read": [46, 46, 47, 111, 117, 116]});
    Backend::External(Adapter {
        command: vec![
            "python3".into(),
            "-c".into(),
            format!(
                "import json,sys; r=json.load(sys.stdin); responses=json.loads({:?}); assert r['operation']=='resolve' or r['arguments']['revision']=='revision-42'; print(json.dumps({{'version':1,'result':responses[r['operation']]}}))",
                responses.to_string()
            ),
        ],
    })
}

#[test]
fn external_revision_export_resolves_opaque_ids_and_rejects_unsafe_trees() {
    let root = tempfile::tempdir().unwrap();
    let file = json!({"path": "file", "kind": "file", "object": "blob-1"});
    let backend = export_adapter(json!([file]));
    let output = backend.source(root.path()).export_revision("tip").unwrap();
    assert_eq!(fs::read(output.path().join("file")).unwrap(), b"../out");
    for entry in [
        json!({"path": ".git/config", "kind": "file", "object": "b"}),
        json!({"path": "../out", "kind": "file", "object": "b"}),
        json!({"path": "sub", "kind": "submodule", "object": "r"}),
    ] {
        assert!(
            export_adapter(json!([entry]))
                .source(root.path())
                .export_revision("tip")
                .is_err()
        );
    }
    let link = json!({"path": "link", "kind": "symlink", "object": "b"});
    let child = json!({"path": "link/child", "kind": "file", "object": "b"});
    assert!(
        export_adapter(json!([link, child]))
            .source(root.path())
            .export_revision("tip")
            .is_err()
    );
}

#[test]
fn private_observation_preserves_selected_identity_and_rejects_invalid_state() {
    let root = tempfile::tempdir().unwrap();
    let (_, candidate) = revisions(root.path());
    let backend = Backend::External(example());
    let before = contents(root.path());
    let observed = backend.source(root.path()).observe().unwrap();
    assert_eq!(observed.backend, backend);
    assert_eq!(observed.revision, candidate);
    assert_eq!(observed.branch, "default");
    assert!(observed.status.contains("new name"));
    assert!(!observed.merge_in_progress && !observed.rebase_in_progress);
    assert_eq!(contents(root.path()), before);
    let valid = json!({"branch":"team/main", "revision":"revision-42", "status":"", "merge_in_progress":false, "rebase_in_progress":false});
    for (field, value) in [
        ("revision", json!("r1\nr2")),
        ("merge_in_progress", json!("false")),
        ("backend", json!("git")),
    ] {
        let mut invalid = valid.clone();
        invalid[field] = value;
        assert!(
            reply(json!({"version":1,"result":invalid}))
                .observe(root.path())
                .is_err()
        );
    }
    assert_eq!(
        reply(json!({"version":1,"result":valid}))
            .observe(root.path())
            .unwrap()
            .revision,
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
fn feature_start_rejects_success_without_the_requested_branch() {
    let root = tempfile::tempdir().unwrap();
    let state = json!({"branch":"main", "revision":"r1", "status":"", "merge_in_progress":false, "rebase_in_progress":false});
    let script = format!(
        "import json,sys; request=json.load(sys.stdin); state=json.loads({:?}); result=state if request['operation']=='observe' else None; print(json.dumps({{'version':1,'result':result}}))",
        state.to_string()
    );
    let adapter = Adapter {
        command: vec!["python3".into(), "-c".into(), script],
    };
    let expected = adapter.observe(root.path()).unwrap();
    let error = adapter
        .start_feature(root.path(), "task/new", &expected)
        .unwrap_err();
    assert!(error.to_string().contains("did not establish"));
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

#[test]
fn selected_external_backend_resolves_through_its_adapter() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::External(reply(json!({"version": 1, "result": "revision-42"})));
    assert_eq!(
        backend.source(root.path()).resolve("tip").unwrap(),
        "revision-42"
    );
}
