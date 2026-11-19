// DECISION: D218
use super::{commit, committed, installed, read, start};
use crate::shared::testing::{configure, git, invoke};
use std::{fs, path::Path};

const PIN: &str = "stable.txt";
const PROPOSED: &str = ".cache/runtime/proposal/stable.txt";
const INSTALLED: &str = ".git/installed";
const INSTALLER: &str = r#"setup:
  purpose: Record the installed revision once the test allows it
  argv: [sh, -c, 'test -f .git/allow && printf "%s\n" "$1" > .git/installed', setup]
  accepts_args: true
"#;

fn head(root: &Path) -> String {
    git(root, &["rev-parse", "HEAD"])
}

fn unhooked(root: &Path, message: &str) {
    git(root, &["add", "."]);
    let unhooked = ["-c", "core.hooksPath=/dev/null"];
    git(root, &[&unhooked[..], &["commit", "-qm", message]].concat());
}

fn pinned(root: &Path, settings: &str) -> String {
    configure(root, "git", settings);
    let first = head(root);
    fs::write(root.join(PIN), format!("{first}\n")).unwrap();
    unhooked(root, "pin");
    first
}

#[test]
fn a_moved_pin_commits_only_when_the_user_approved_a_forward_revision() {
    let (_temp, root) = installed();
    let first = pinned(&root, "pin: stable.txt\n");
    start(&root, "pin");
    commit(&root, "product.txt", "product");
    let product = head(&root);
    let moved = |revision: &str| fs::write(root.join(PIN), format!("{revision}\n")).unwrap();
    moved(&product);
    let unapproved = committed(&root, "unapproved");
    assert_eq!(unapproved.code, Some(1), "{}", unapproved.output());
    assert!(unapproved.stderr.contains(PIN), "{}", unapproved.stderr);
    let approved = invoke(&root, &["approve"]);
    assert_eq!(approved.code, Some(0), "{}", approved.output());
    let forward = committed(&root, "forward");
    assert_eq!(forward.code, Some(0), "{}", forward.output());
    moved(&first);
    let approved = invoke(&root, &["approve"]);
    assert_eq!(approved.code, Some(0), "{}", approved.output());
    let backward = committed(&root, "backward");
    assert_eq!(backward.code, Some(1), "{}", backward.output());
    assert!(backward.stderr.contains(PIN), "{}", backward.stderr);
}

#[test]
fn approve_installs_a_proposed_pin_before_writing_it() {
    let (_temp, root) = installed();
    configure(&root, "commands", INSTALLER);
    let first = pinned(&root, "pin: stable.txt\ninstall: setup\n");
    start(&root, "pin");
    fs::create_dir_all(root.join("Project")).unwrap();
    fs::write(root.join("Project/product.txt"), "product").unwrap();
    unhooked(&root, "product");
    let product = head(&root);
    let proposed = invoke(&root, &["propose-pin", "HEAD"]);
    assert_eq!(proposed.code, Some(0), "{}", proposed.output());
    for named in [product.as_str(), "Project/product.txt", "! just approve"] {
        assert!(proposed.stdout.contains(named), "{}", proposed.stdout);
    }
    assert!(!proposed.stdout.contains(PIN), "{}", proposed.stdout);
    assert_eq!(read(&root, PIN).trim(), first);
    let refused = invoke(&root, &["approve"]);
    assert_ne!(refused.code, Some(0), "{}", refused.output());
    assert_eq!(read(&root, PIN).trim(), first);
    assert_eq!(read(&root, PROPOSED).trim(), product);
    assert!(!root.join(INSTALLED).exists());
    fs::write(root.join(".git/allow"), "").unwrap();
    let approved = invoke(&root, &["approve"]);
    assert_eq!(approved.code, Some(0), "{}", approved.output());
    assert_eq!(read(&root, INSTALLED).trim(), product);
    assert_eq!(read(&root, PIN).trim(), product);
    assert!(!root.join(PROPOSED).exists());
    unhooked(&root, "pinned");
    let backward = invoke(&root, &["propose-pin", &first]);
    assert_ne!(backward.code, Some(0), "{}", backward.output());
    assert!(
        backward.stderr.contains("does not descend"),
        "{}",
        backward.stderr
    );
}
