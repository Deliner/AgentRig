// DECISION: D218
use super::{PROBE_CHECK, commit, committed, installed, read, start};
use crate::shared::testing::{configure, git, invoke, run, runtime};
use std::{
    fs,
    path::{Path, PathBuf},
};

const PIN: &str = "stable.txt";
const PLAN: &str = "Ledger/Plan.md";
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

fn planned(root: &Path, status: &str) {
    let plan = format!(
        "# Plan\n\n| ID | Status | Depends on | Feature | User capability |\n| --- | --- | --- | --- | --- |\n| [P001](Plan/001.md) | {status} | - | Feature | Capability |\n"
    );
    fs::write(root.join(PLAN), plan).unwrap();
    unhooked(root, status);
}

struct Consented {
    _temp: tempfile::TempDir,
    root: PathBuf,
    first: String,
}

fn consented_product(settings: &str, status: &str) -> Consented {
    let (temp, root) = installed();
    fs::write(root.join(".agentrig/checks.yaml"), PROBE_CHECK).unwrap();
    configure(&root, "commands", INSTALLER);
    let first = pinned(&root, settings);
    planned(&root, status);
    start(&root, "consent");
    fs::create_dir_all(root.join("Project")).unwrap();
    fs::write(root.join("Project/product.txt"), "product").unwrap();
    unhooked(&root, "product");
    let given = invoke(&root, &["consent"]);
    assert_eq!(given.code, Some(0), "{}", given.output());
    Consented {
        _temp: temp,
        root,
        first,
    }
}

// DECISION: D226
#[test]
fn under_consent_propose_pin_installs_and_pins_a_gated_head_and_lists_it() {
    let Consented { _temp, root, .. } =
        consented_product("pin: stable.txt\ninstall: setup\n", "active");
    let product = head(&root);
    fs::write(root.join(".git/allow"), "").unwrap();
    let pinned = invoke(&root, &["propose-pin", "HEAD"]);
    assert_eq!(pinned.code, Some(0), "{}", pinned.output());
    assert_eq!(read(&root, INSTALLED).trim(), product);
    assert_eq!(read(&root, PIN).trim(), product);
    assert!(!root.join(PROPOSED).exists());
    let accepted = committed(&root, "pinned");
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
    let listed = invoke(&root, &["consented-pins"]);
    assert_eq!(listed.code, Some(0), "{}", listed.output());
    for named in [product.as_str(), "Project/product.txt"] {
        assert!(listed.stdout.contains(named), "{}", listed.stdout);
    }
}

// DECISION: D226
#[test]
fn under_consent_propose_pin_refuses_another_revision_a_dirty_tree_and_a_failed_gate() {
    let Consented { _temp, root, first } = consented_product("pin: stable.txt\n", "pending");
    let earlier = invoke(&root, &["propose-pin", &first]);
    assert_ne!(earlier.code, Some(0), "{}", earlier.output());
    assert!(earlier.stderr.contains("only HEAD"), "{}", earlier.stderr);
    fs::write(root.join("Project/product.txt"), "uncommitted").unwrap();
    let dirty = invoke(&root, &["propose-pin", "HEAD"]);
    assert_ne!(dirty.code, Some(0), "{}", dirty.output());
    assert!(
        dirty.stderr.contains("working tree must be clean"),
        "{}",
        dirty.stderr
    );
    git(&root, &["checkout", "--", "Project/product.txt"]);
    let blocked = run(
        runtime(&root, &["propose-pin", "HEAD"]).env("BLOCK_DELIVERY", "1"),
        "",
    );
    assert_eq!(blocked.code, Some(1), "{}", blocked.output());
    assert!(
        blocked.stderr.contains("ERROR [probe]"),
        "{}",
        blocked.stderr
    );
    assert_eq!(read(&root, PIN).trim(), first);
}

// DECISION: D226
#[test]
fn without_consent_after_its_items_or_once_revoked_propose_pin_prepares_a_proposal() {
    let Consented { _temp, root, first } = consented_product("pin: stable.txt\n", "active");
    let proposes = |root: &Path| {
        let proposed = invoke(root, &["propose-pin", "HEAD"]);
        assert_eq!(proposed.code, Some(0), "{}", proposed.output());
        assert!(
            proposed.stdout.contains("! just approve"),
            "{}",
            proposed.stdout
        );
        assert_eq!(read(root, PIN).trim(), first);
    };
    let revoked = invoke(&root, &["revoke-consent"]);
    assert_eq!(revoked.code, Some(0), "{}", revoked.output());
    proposes(&root);
    let given = invoke(&root, &["consent"]);
    assert_eq!(given.code, Some(0), "{}", given.output());
    planned(&root, "complete");
    proposes(&root);
    let refused = invoke(&root, &["consent"]);
    assert_ne!(refused.code, Some(0), "{}", refused.output());
}
