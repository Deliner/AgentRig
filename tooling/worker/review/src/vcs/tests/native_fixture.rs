use review_runner::vcs::{Kind, Repository};
use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

pub fn command(root: &Path, program: &str, args: &[&str]) {
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

pub fn initialize(root: &Path, kind: Kind) {
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

pub fn commit(root: &Path, kind: Kind) -> String {
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

pub fn revisions(root: &Path, kind: Kind) -> (String, String) {
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
