use review_runner::{config::Repository, snapshot};
use std::{fs, path::Path, process::Command};

fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args([
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
        ])
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap().trim().into()
}
fn commit(root: &Path) -> String {
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "fixture"]);
    git(root, &["rev-parse", "HEAD"])
}
fn setup(root: &Path) -> String {
    git(root, &["init", "-q"]);
    fs::create_dir(root.join("src")).unwrap();
    fs::create_dir(root.join("private")).unwrap();
    fs::write(root.join("src/a.txt"), "before").unwrap();
    fs::write(root.join("private/hidden.txt"), "not exposed").unwrap();
    commit(root)
}
fn scope() -> Repository {
    Repository {
        visible_paths: vec!["src/**".into()],
        contract_paths: vec![],
    }
}
#[test]
fn snapshot_reads_commit_not_live_checkout() {
    let root = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    let base = setup(root.path());
    fs::write(root.path().join("src/a.txt"), "candidate").unwrap();
    let candidate = commit(root.path());
    fs::write(root.path().join("src/a.txt"), "uncommitted").unwrap();
    fs::write(root.path().join("src/untracked.txt"), "untracked").unwrap();
    let path = output.path().join("snapshot");
    let snapshot = snapshot::prepare(root.path(), (&base, &candidate), &scope(), &path).unwrap();
    assert_eq!(snapshot.base, base);
    assert_eq!(snapshot.candidate, candidate);
    assert_eq!(
        fs::read_to_string(path.join("src/a.txt")).unwrap(),
        "candidate"
    );
    assert_eq!(snapshot.manifest.len(), 1);
    assert!(!path.join(".git").exists());
    assert!(!path.join("private").exists());
    assert!(snapshot.diff.contains("-before"));
    assert!(snapshot.diff.contains("+candidate"));
}
#[test]
fn deletion_and_rename_cannot_bypass_visibility() {
    for rename in [false, true] {
        let root = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let base = setup(root.path());
        if rename {
            fs::rename(
                root.path().join("private/hidden.txt"),
                root.path().join("src/moved.txt"),
            )
            .unwrap();
        } else {
            fs::remove_file(root.path().join("private/hidden.txt")).unwrap();
        }
        let candidate = commit(root.path());
        let result = snapshot::prepare(
            root.path(),
            (&base, &candidate),
            &scope(),
            &output.path().join("snapshot"),
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("outside visible_paths")
        );
    }
}
#[test]
fn sensitive_files_and_symlinks_are_rejected() {
    for name in [".env", "key.txt", "link"] {
        let root = tempfile::tempdir().unwrap();
        let output = tempfile::tempdir().unwrap();
        let base = setup(root.path());
        let path = root.path().join("src").join(name);
        let link = name == "link";
        if link {
            std::os::unix::fs::symlink("/etc/passwd", &path).unwrap();
        } else {
            fs::write(path, "-----BEGIN PRIVATE KEY-----").unwrap();
        }
        let candidate = commit(root.path());
        assert!(
            snapshot::prepare(
                root.path(),
                (&base, &candidate),
                &scope(),
                &output.path().join("snapshot")
            )
            .is_err()
        );
    }
}
