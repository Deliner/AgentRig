mod fixture;
use super::{FileKind, Repository, Revisions};
use fixture::{command, commit, initialize, revisions};
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

#[test]
fn revisions_are_read_exactly_and_preserve_worktrees() {
    {
        let root = tempfile::tempdir().unwrap();
        let (base, candidate) = revisions(root.path());
        let repository = Repository::new(root.path());
        let (renamed, binary) = (Path::new("new name"), Path::new("binary"));
        let range = Revisions {
            from: &base,
            to: &candidate,
        };
        assert_eq!(repository.read(&candidate, renamed).unwrap(), b"before\n");
        assert_eq!(repository.read(&base, binary).unwrap(), [0, 255, 10]);
        assert_eq!(repository.read(&candidate, binary).unwrap(), [0, 255, 20]);
        assert_eq!(
            repository.changed_paths(range).unwrap(),
            ["binary", "new name", "old name"]
        );
        let tree = repository.tree(&candidate).unwrap();
        assert_eq!(tree["executable"].kind, FileKind::Executable);
        assert_eq!(tree["link"].kind, FileKind::Symlink);
        assert_eq!(tree["new name"].kind, FileKind::File);
        assert!(!tree["binary"].object.is_empty());
        let diff = repository.diff(range).unwrap();
        assert!(diff.contains("new name") && diff.contains("old name") && diff.contains("binary"));
        assert_eq!(
            fs::read(root.path().join("new name")).unwrap(),
            b"uncommitted\n"
        );
        assert!(repository.resolve("missing-reference").is_err());
    }
}

#[test]
fn heads_distinguish_unborn_repositories_from_committed_revisions() {
    {
        let root = tempfile::tempdir().unwrap();
        initialize(root.path());
        let repository = Repository::new(root.path());
        assert_eq!(repository.head().unwrap(), None);
        fs::write(root.path().join("value"), "committed").unwrap();
        let revision = commit(root.path());
        assert_eq!(repository.head().unwrap(), Some(revision));
    }
}

#[test]
fn full_revision_exports_preserve_project_inputs_and_file_kinds() {
    {
        let root = tempfile::tempdir().unwrap();
        let (base, _) = revisions(root.path());
        fs::create_dir_all(root.path().join(".agents/skills/example")).unwrap();
        fs::write(
            root.path().join(".agents/skills/example/SKILL.md"),
            "instructions",
        )
        .unwrap();
        fs::write(root.path().join("agentrig.yaml"), "version: 1\n").unwrap();
        let candidate = commit(root.path());
        fs::write(root.path().join("agentrig.yaml"), "dirty").unwrap();
        let repository = Repository::new(root.path());
        let export = repository.export_revision(&candidate).unwrap();
        assert_eq!(
            fs::read(export.path().join("agentrig.yaml")).unwrap(),
            b"version: 1\n"
        );
        assert_eq!(
            fs::read(export.path().join(".agents/skills/example/SKILL.md")).unwrap(),
            b"instructions"
        );
        assert_exported_file_kinds(export.path());
        assert!(!export.path().join("old name").exists());
        assert!(!export.path().join(".git").exists());
        let previous = repository.export_revision(&base).unwrap();
        assert_eq!(fs::read(previous.path().join("link")).unwrap(), b"before\n");
        assert!(!previous.path().join("new name").exists());
        assert_eq!(
            fs::read(root.path().join("agentrig.yaml")).unwrap(),
            b"dirty"
        );
    }
}

fn assert_exported_file_kinds(root: &Path) {
    assert_eq!(fs::read(root.join("binary")).unwrap(), [0, 255, 20]);
    assert_eq!(
        fs::read_link(root.join("link")).unwrap(),
        Path::new("old name")
    );
    assert_eq!(
        fs::metadata(root.join("executable"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o755
    );
}

#[test]
fn index_export_preserves_staged_content() {
    let root = tempfile::tempdir().unwrap();
    revisions(root.path());
    fs::write(root.path().join("new name"), "staged").unwrap();
    command(root.path(), "git", &["add", "new name"]);
    fs::write(root.path().join("new name"), "unstaged").unwrap();
    let repository = Repository::new(root.path());
    let before = repository.index_entries().unwrap();
    let export = repository.export_index().unwrap();
    assert_eq!(fs::read(export.path().join("new name")).unwrap(), b"staged");
    assert_eq!(fs::read(root.path().join("new name")).unwrap(), b"unstaged");
    assert_eq!(repository.index_entries().unwrap(), before);
}

#[test]
fn full_revision_export_rejects_control_metadata_and_submodules() {
    let root = tempfile::tempdir().unwrap();
    let (_, revision) = revisions(root.path());
    command(
        root.path(),
        "git",
        &[
            "update-index",
            "--add",
            "--cacheinfo",
            &format!("160000,{revision},dependency"),
        ],
    );
    command(root.path(), "git", &["commit", "-qm", "submodule"]);
    let repository = Repository::new(root.path());
    let error = repository.export_revision("HEAD").unwrap_err();
    assert!(
        error
            .to_string()
            .contains("cannot include submodule dependency")
    );
    command(
        root.path(),
        "git",
        &["update-index", "--force-remove", "dependency"],
    );
}

#[test]
fn revision_parents_include_both_merge_sides_and_omit_null_roots() {
    {
        let root = tempfile::tempdir().unwrap();
        initialize(root.path());
        fs::write(root.path().join("base"), "base").unwrap();
        let base = commit(root.path());
        fs::write(root.path().join("left"), "left").unwrap();
        let left = commit(root.path());
        command(root.path(), "git", &["checkout", "-q", &base]);
        fs::write(root.path().join("right"), "right").unwrap();
        let right = commit(root.path());
        command(root.path(), "git", &["merge", "--no-commit", &left]);
        let merged = commit(root.path());
        let repository = Repository::new(root.path());
        assert!(repository.parents(&base).unwrap().is_empty());
        assert_eq!(repository.parents(&left).unwrap(), [base]);
        assert_eq!(repository.parents(&merged).unwrap(), [right, left]);
    }
}
