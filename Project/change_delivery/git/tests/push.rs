// DECISION: D179
use super::{installed, merge, start};
use crate::shared::testing::{Run, commit, configure, git, git_run, invoke};
use std::{fs, path::Path, path::PathBuf};
use tempfile::TempDir;

fn bare() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().canonicalize().unwrap();
    git(&path, &["init", "-q", "--bare"]);
    (temp, path)
}

fn published(root: &Path, remote: &Path) {
    git(root, &["remote", "add", "origin", remote.to_str().unwrap()]);
    configure(root, "git", "remote: origin\n");
    let unhooked = ["-c", "core.hooksPath=/dev/null"];
    git(
        root,
        &[&unhooked[..], &["commit", "-qam", "publish"]].concat(),
    );
}

fn tip(repository: &Path, branch: &str) -> String {
    git(repository, &["rev-parse", &format!("refs/heads/{branch}")])
}

fn committed(root: &Path, name: &str) -> Run {
    fs::write(root.join(name), name).unwrap();
    git(root, &["add", name]);
    git_run(root, &["commit", "-qm", name])
}

fn foreign(remote: &Path, branch: &str) -> (TempDir, String) {
    let temp = tempfile::tempdir().unwrap();
    let clone = temp.path().canonicalize().unwrap();
    let source = remote.to_str().unwrap();
    git(&clone, &["clone", "-q", "-b", branch, source, "."]);
    let identity = [
        "-c",
        "user.name=Other",
        "-c",
        "user.email=other@example.invalid",
    ];
    let change = ["commit", "--allow-empty", "-qm", "foreign"];
    git(&clone, &[&identity[..], &change].concat());
    git(&clone, &["push", "-q", "origin", branch]);
    let pushed = git(&clone, &["rev-parse", "HEAD"]);
    (temp, pushed)
}

#[test]
fn merge_pushes_the_base_and_the_rebuilt_feature_only_when_a_remote_is_configured() {
    let (_temp, root) = installed();
    let (_remote_temp, remote) = bare();
    git(
        &root,
        &["remote", "add", "origin", remote.to_str().unwrap()],
    );
    start(&root, "quiet");
    commit(&root, "quiet.txt", "quiet");
    merge(&root);
    assert_eq!(git(&remote, &["for-each-ref"]), "");
    git(&root, &["remote", "remove", "origin"]);
    published(&root, &remote);
    start(&root, "product");
    commit(&root, "product.txt", "product");
    let before = git(&root, &["rev-parse", "HEAD"]);
    git(&root, &["switch", "trunk"]);
    start(&root, "concurrent");
    commit(&root, "concurrent.txt", "concurrent");
    merge(&root);
    git(&root, &["switch", "task/product"]);
    let merged = invoke(&root, &["feature-merge"]);
    assert_eq!(merged.code, Some(0), "{}", merged.output());
    let report = "pushed trunk, task/product to origin";
    assert!(merged.stdout.contains(report), "{}", merged.output());
    assert_ne!(tip(&root, "task/product"), before);
    for branch in ["trunk", "task/product", "task/concurrent"] {
        assert_eq!(tip(&remote, branch), tip(&root, branch), "{branch}");
    }
}

#[test]
fn commits_push_the_feature_branch_without_overwriting_another_clone() {
    let (_temp, root) = installed();
    let (_remote_temp, remote) = bare();
    published(&root, &remote);
    start(&root, "product");
    commit(&root, "product.txt", "product");
    assert_eq!(tip(&remote, "task/product"), tip(&root, "task/product"));
    git(&root, &["switch", "trunk"]);
    start(&root, "concurrent");
    commit(&root, "concurrent.txt", "concurrent");
    merge(&root);
    git(&root, &["switch", "task/product"]);
    git(&root, &["rebase", "-q", "trunk"]);
    commit(&root, "rebuilt.txt", "rebuilt");
    assert_eq!(tip(&remote, "task/product"), tip(&root, "task/product"));
    let (_clone, other) = foreign(&remote, "task/product");
    let refused = committed(&root, "later.txt");
    assert_eq!(refused.code, Some(0), "{}", refused.output());
    let warning = "WARNING: commit kept; task/product not pushed to origin";
    assert!(refused.stderr.contains(warning), "{}", refused.output());
    assert_eq!(git(&root, &["log", "-1", "--format=%s"]), "later.txt");
    assert_eq!(tip(&remote, "task/product"), other);
}

#[test]
fn unreachable_remote_keeps_the_commit_and_the_merge_and_names_unpushed_branches() {
    let (_temp, root) = installed();
    let (_remote_temp, remote) = bare();
    published(&root, &remote.join("absent.git"));
    start(&root, "product");
    let warned = committed(&root, "product.txt");
    assert_eq!(warned.code, Some(0), "{}", warned.output());
    let warning = "WARNING: commit kept; task/product not pushed to origin: git failed: fatal:";
    assert!(warned.stderr.contains(warning), "{}", warned.output());
    assert_eq!(git(&root, &["log", "-1", "--format=%s"]), "product.txt");
    let merged = invoke(&root, &["feature-merge"]);
    assert_eq!(merged.code, Some(1), "{}", merged.output());
    assert_eq!(
        git(&root, &["rev-parse", "trunk^2"]),
        tip(&root, "task/product")
    );
    for unpushed in [
        "merge kept; trunk not pushed to origin: git failed: fatal:",
        "merge kept; task/product not pushed to origin: git failed: fatal:",
        "retry: git push origin refs/heads/trunk:refs/heads/trunk",
    ] {
        assert!(merged.stderr.contains(unpushed), "{}", merged.output());
    }
}

#[test]
fn merge_keeps_a_feature_another_clone_changed_on_the_remote() {
    let (_temp, root) = installed();
    let (_remote_temp, remote) = bare();
    published(&root, &remote);
    start(&root, "product");
    commit(&root, "product.txt", "product");
    let (_clone, other) = foreign(&remote, "task/product");
    let merged = invoke(&root, &["feature-merge"]);
    assert_eq!(merged.code, Some(1), "{}", merged.output());
    assert!(
        merged.stdout.contains("pushed trunk to origin"),
        "{}",
        merged.output()
    );
    let refused = "merge kept; task/product not pushed to origin: git failed:";
    let retry = "retry: git push '--force-with-lease=refs/heads/task/product' origin refs/heads/task/product:refs/heads/task/product";
    for expected in [refused, retry] {
        assert!(merged.stderr.contains(expected), "{}", merged.output());
    }
    assert_eq!(
        git(&root, &["rev-parse", "trunk^2"]),
        tip(&root, "task/product")
    );
    assert_eq!(tip(&remote, "trunk"), tip(&root, "trunk"));
    assert_eq!(tip(&remote, "task/product"), other);
}
