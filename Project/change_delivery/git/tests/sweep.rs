// DECISION: D233
use super::{approve_commands, commit, installed, start};
use crate::shared::testing::{Run, configure, git, git_run, invoke, run, runtime};
use serde_json::Value;
use std::{
    fs::{self, Permissions},
    mem::DropGuard,
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const IGNORED: &str = "build/\n.env*\n";
const WAIT: &str = r#"wait:
  purpose: Wait as a task job
  argv: [sh, -c, 'echo ready; exec sleep 60']
"#;
const OWNER: &str = "other-owner";
const STATE: [&str; 4] = [
    ".cache/runtime/approved",
    ".cache/runtime/consent",
    ".cache/runtime/proposal/stable.txt",
    ".cache/runtime/compaction",
];

fn merged(root: &Path) -> Run {
    invoke(root, &["feature-merge"])
}

fn succeeded(root: &Path) -> Run {
    let merged = merged(root);
    assert_eq!(merged.code, Some(0), "{}", merged.output());
    merged
}

fn approve(root: &Path) {
    let approved = invoke(root, &["approve"]);
    assert_eq!(approved.code, Some(0), "{}", approved.output());
}

fn unhooked(root: &Path, message: &str) {
    git(root, &["add", "."]);
    git(
        root,
        &["-c", "core.hooksPath=/dev/null", "commit", "-qm", message],
    );
}

fn owned(root: &Path, args: &[&str]) -> Value {
    let result = run(runtime(root, args).env("WORKER_OWNER", OWNER), "");
    assert_eq!(result.code, Some(0), "{}", result.output());
    serde_json::from_str(&result.stdout).unwrap()
}

fn running(root: &Path) -> String {
    let started = owned(root, &["job-start", "wait"]);
    let identifier = started["run_id"].as_str().unwrap().to_owned();
    let deadline = Instant::now() + Duration::from_secs(5);
    while owned(root, &["job-status", &identifier])["state"] != "running" {
        assert!(Instant::now() < deadline, "{identifier} did not start");
        std::thread::sleep(Duration::from_millis(10));
    }
    identifier
}

fn listed(root: &Path) -> Vec<String> {
    let rows = owned(root, &["jobs"]);
    let rows = rows.as_array().unwrap();
    rows.iter()
        .map(|row| row["run_id"].as_str().unwrap().to_owned())
        .collect()
}

fn completed(root: &Path) -> String {
    let probed = invoke(root, &["run", "probe"]);
    assert_eq!(probed.code, Some(0), "{}", probed.output());
    listed(root).pop().unwrap()
}

fn built(root: &Path, outside: &Path) {
    fs::write(outside.join("target.txt"), "outside").unwrap();
    fs::write(root.join(".gitignore"), IGNORED).unwrap();
    fs::create_dir(root.join("build")).unwrap();
    fs::write(root.join("build/out.bin"), vec![0_u8; 4096]).unwrap();
    fs::write(root.join("build/kept.txt"), "tracked").unwrap();
    symlink(outside, root.join("build/linked")).unwrap();
    git(root, &["add", "-f", "build/kept.txt"]);
}

fn pending(root: &Path) {
    fs::write(root.join(".cache/runtime/consent"), "{}").unwrap();
    fs::create_dir_all(root.join(".cache/runtime/proposal")).unwrap();
    fs::write(root.join(".cache/runtime/proposal/stable.txt"), "proposed").unwrap();
}

#[test]
fn merge_removes_ignored_files_and_finished_runs_but_keeps_state_tracked_and_outside_files() {
    let (_temp, root) = installed();
    let outside = tempfile::tempdir().unwrap();
    start(&root, "sweep");
    approve_commands(&root, WAIT);
    built(&root, outside.path());
    commit(&root, "work.txt", "feature");
    let finished = completed(&root);
    let live = DropGuard::new(running(&root), |identifier| {
        let mut stop = runtime(&root, &["job-stop", &identifier]);
        run(stop.env("WORKER_OWNER", OWNER), "");
    });
    pending(&root);
    let merged = succeeded(&root);
    let named = [
        "removed ignored files, freeing",
        "build/out.bin (4.0 KiB)",
        "finished runs",
    ];
    for name in named {
        assert!(merged.stdout.contains(name), "{name}: {}", merged.stdout);
    }
    assert!(!root.join("build/out.bin").exists());
    assert!(!root.join("build/linked").exists());
    let tracked = fs::read_to_string(root.join("build/kept.txt")).unwrap();
    assert_eq!(tracked, "tracked");
    let target = fs::read_to_string(outside.path().join("target.txt")).unwrap();
    assert_eq!(target, "outside");
    for state in STATE {
        assert!(root.join(state).exists(), "{state}");
    }
    let remaining = listed(&root);
    assert!(remaining.contains(&*live), "{remaining:?}");
    assert!(!remaining.contains(&finished), "{remaining:?}");
    assert_eq!(git(&root, &["status", "--porcelain"]), "");
}

#[test]
fn a_kept_path_survives_until_it_leaves_the_list() {
    let (_temp, root) = installed();
    start(&root, "keep");
    fs::write(root.join(".gitignore"), IGNORED).unwrap();
    configure(&root, "git", "keep: ['.env*']\n");
    approve(&root);
    fs::write(root.join(".env"), "TOKEN=local\n").unwrap();
    commit(&root, "work.txt", "feature");
    succeeded(&root);
    assert!(root.join(".env").exists());
    start(&root, "forget");
    configure(&root, "git", "keep: []\n");
    approve(&root);
    commit(&root, "more.txt", "more");
    let merged = succeeded(&root);
    assert!(!root.join(".env").exists());
    assert!(merged.stdout.contains("  .env ("), "{}", merged.stdout);
}

#[test]
fn only_the_pinned_runtime_remains() {
    let (_temp, root) = installed();
    configure(&root, "git", "pin: stable.txt\n");
    let pinned = git(&root, &["rev-parse", "HEAD"]);
    fs::write(root.join("stable.txt"), format!("{pinned}\n")).unwrap();
    unhooked(&root, "pin");
    let development = root.join(".cache/development");
    let older = "0".repeat(40);
    for name in [pinned.as_str(), older.as_str(), "build"] {
        fs::create_dir_all(development.join(name).join("bin")).unwrap();
        fs::write(development.join(name).join("bin/agentrig"), name).unwrap();
    }
    start(&root, "runtime");
    commit(&root, "work.txt", "feature");
    succeeded(&root);
    let remaining: Vec<PathBuf> = fs::read_dir(&development)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    assert_eq!(remaining, [development.join(&pinned)]);
}

#[test]
fn an_unreadable_pin_keeps_every_ignored_file() {
    let (_temp, root) = installed();
    configure(&root, "git", "pin: stable.txt\n");
    unhooked(&root, "pin");
    let ignored = root.join(".cache/generated");
    fs::write(&ignored, "generated").unwrap();
    start(&root, "unpinned");
    commit(&root, "work.txt", "feature");
    let refused = merged(&root);
    assert_eq!(refused.code, Some(1), "{}", refused.output());
    let kept = refused.stderr.contains("ignored files were kept");
    assert!(kept, "{}", refused.stderr);
    assert!(ignored.exists());
}

#[test]
fn an_undeletable_path_leaves_the_merge_done_and_the_next_merge_removes_it() {
    let (_temp, root) = installed();
    let locked = root.join(".cache/locked");
    fs::create_dir(&locked).unwrap();
    fs::write(locked.join("file"), "generated").unwrap();
    let permissions = |mode| fs::set_permissions(&locked, Permissions::from_mode(mode)).unwrap();
    permissions(0o555);
    start(&root, "locked");
    commit(&root, "work.txt", "feature");
    let failed = merged(&root);
    permissions(0o755);
    assert_eq!(failed.code, Some(1), "{}", failed.output());
    let named = [
        ".cache/locked/file",
        "Permission denied",
        "the next merge retries",
    ];
    for name in named {
        assert!(failed.stderr.contains(name), "{name}: {}", failed.stderr);
    }
    let merge = ["merge-base", "--is-ancestor", "task/locked", "trunk"];
    let included = git_run(&root, &merge);
    assert_eq!(included.code, Some(0), "{}", included.output());
    start(&root, "retry");
    commit(&root, "more.txt", "more");
    succeeded(&root);
    assert!(!locked.exists());
}
