use review_runner::vcs::{FileKind, Kind, Repository};
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

fn command(root: &Path, program: &str, args: &[&str]) {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .env("HGRCPATH", "")
        .env("HGRCSKIPREPO", "1")
        .env("HGPLAIN", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn initialize(root: &Path, kind: Kind) {
    match kind {
        Kind::Git => {
            command(root, "git", &["init", "-q"]);
            command(
                root,
                "git",
                &["config", "user.email", "test@example.invalid"],
            );
            command(root, "git", &["config", "user.name", "Test"]);
        }
        Kind::Mercurial => command(root, "hg", &["init"]),
    }
}

fn commit(root: &Path, kind: Kind) -> String {
    let reference = match kind {
        Kind::Git => {
            command(root, "git", &["add", "-A"]);
            command(root, "git", &["commit", "-qm", "fixture"]);
            "HEAD"
        }
        Kind::Mercurial => {
            command(root, "hg", &["addremove"]);
            command(root, "hg", &["commit", "-m", "fixture", "-u", "Test"]);
            "."
        }
    };
    Repository::new(root, kind).resolve(reference).unwrap()
}

fn revisions(root: &Path, kind: Kind) -> (String, String) {
    initialize(root, kind);
    fs::write(root.join("old name"), b"before\n").unwrap();
    fs::write(root.join("binary"), [0, 255, 10]).unwrap();
    fs::write(root.join("executable"), b"run\n").unwrap();
    fs::set_permissions(root.join("executable"), fs::Permissions::from_mode(0o755)).unwrap();
    std::os::unix::fs::symlink("old name", root.join("link")).unwrap();
    let base = commit(root, kind);
    fs::rename(root.join("old name"), root.join("new name")).unwrap();
    fs::write(root.join("binary"), [0, 255, 20]).unwrap();
    let candidate = commit(root, kind);
    fs::write(root.join("new name"), b"uncommitted\n").unwrap();
    (base, candidate)
}

#[test]
fn native_backends_read_exact_revisions_and_preserve_worktrees() {
    for kind in [Kind::Git, Kind::Mercurial] {
        let root = tempfile::tempdir().unwrap();
        let (base, candidate) = revisions(root.path(), kind);
        let repository = Repository::new(root.path(), kind);
        assert_eq!(
            repository.read(&candidate, "new name").unwrap(),
            b"before\n"
        );
        assert_eq!(repository.read(&base, "binary").unwrap(), [0, 255, 10]);
        assert_eq!(repository.read(&candidate, "binary").unwrap(), [0, 255, 20]);
        assert_eq!(
            repository.changed_paths(&base, &candidate).unwrap(),
            ["binary", "new name", "old name"]
        );
        let tree = repository.tree(&candidate).unwrap();
        assert_eq!(tree["executable"].kind, FileKind::Executable);
        assert_eq!(tree["link"].kind, FileKind::Symlink);
        assert_eq!(tree["new name"].kind, FileKind::File);
        assert!(!tree["binary"].object.is_empty());
        let diff = repository.diff(&base, &candidate).unwrap();
        assert!(diff.contains("new name") && diff.contains("old name") && diff.contains("binary"));
        assert_eq!(
            fs::read(root.path().join("new name")).unwrap(),
            b"uncommitted\n"
        );
        assert!(repository.resolve("missing-reference").is_err());
    }
}

#[test]
fn mercurial_rejects_multiple_revisions_and_ignores_repository_commands() {
    let root = tempfile::tempdir().unwrap();
    let (_, candidate) = revisions(root.path(), Kind::Mercurial);
    fs::write(
        root.path().join(".hg/hgrc"),
        "[alias]\nlog = !touch executed\n[hooks]\npre-cat = touch executed\n",
    )
    .unwrap();
    let repository = Repository::new(root.path(), Kind::Mercurial);
    assert_eq!(repository.resolve(".").unwrap(), candidate);
    assert!(repository.resolve("all()").is_err());
    assert!(repository.read(&candidate, "binary").is_ok());
    assert!(!root.path().join("executed").exists());
}
