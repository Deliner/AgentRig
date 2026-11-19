// DECISION: D213
use super::{commit, installed, read, start};
use crate::shared::testing::{Run, configure, describe_state, git, git_run, invoke};
use std::{fs, path::Path};

const CATALOG: &str = ".agentrig/commands.yaml";
const PROPOSAL: &str = ".cache/runtime/proposal";
const RECORD: &str = ".cache/runtime/approved";
const ADDED: &str = "extra:\n  purpose: Run an approved addition\n  argv: [sh, -c, 'true']\n";

fn committed(root: &Path, message: &str) -> Run {
    describe_state(root, "A test VAC.");
    git(root, &["add", "."]);
    git_run(root, &["commit", "-qm", message])
}

#[test]
fn a_change_of_enforcement_commits_only_with_the_content_the_user_approved() {
    let (_temp, root) = installed();
    start(&root, "enforcement");
    configure(&root, "commands", ADDED);
    let unapproved = committed(&root, "unapproved");
    assert_eq!(unapproved.code, Some(1), "{}", unapproved.output());
    let skill = root.join(".agentrig/skills/propose-protected-change/SKILL.md");
    for named in [CATALOG, skill.to_str().unwrap()] {
        assert!(unapproved.stderr.contains(named), "{}", unapproved.stderr);
    }
    let approved = invoke(&root, &["approve"]);
    assert_eq!(approved.code, Some(0), "{}", approved.output());
    assert!(approved.stdout.contains(CATALOG), "{}", approved.stdout);
    let accepted = committed(&root, "approved");
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
    configure(
        &root,
        "commands",
        "later:\n  purpose: Run later\n  argv: [sh, -c, 'true']\n",
    );
    let changed = committed(&root, "changed after approval");
    assert_eq!(changed.code, Some(1), "{}", changed.output());
    fs::write(root.join(RECORD), "{").unwrap();
    let unreadable = committed(&root, "unreadable approval");
    assert_ne!(unreadable.code, Some(0), "{}", unreadable.output());
    assert!(unreadable.stderr.contains(RECORD), "{}", unreadable.stderr);
}

#[test]
fn approve_applies_a_valid_proposal_and_restores_the_files_after_an_invalid_one() {
    let (_temp, root) = installed();
    start(&root, "proposal");
    let original = read(&root, CATALOG);
    let proposed = root.join(PROPOSAL).join(CATALOG);
    fs::create_dir_all(proposed.parent().unwrap()).unwrap();
    fs::write(&proposed, "read:\n  argv: []\n").unwrap();
    let invalid = invoke(&root, &["approve"]);
    assert_ne!(invalid.code, Some(0), "{}", invalid.output());
    assert!(invalid.stderr.contains("restored"), "{}", invalid.stderr);
    assert_eq!(read(&root, CATALOG), original);
    assert!(proposed.exists());
    let extended = format!("{original}{ADDED}");
    fs::write(&proposed, &extended).unwrap();
    let applied = invoke(&root, &["approve"]);
    assert_eq!(applied.code, Some(0), "{}", applied.output());
    assert_eq!(read(&root, CATALOG), extended);
    assert!(!root.join(PROPOSAL).exists());
    let accepted = committed(&root, "proposed");
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
}

#[test]
fn a_staged_pin_commits_only_when_it_descends_from_the_committed_one() {
    let (_temp, root) = installed();
    let first = git(&root, &["rev-parse", "HEAD"]);
    configure(&root, "git", "pin: stable.txt\n");
    fs::write(root.join("stable.txt"), format!("{first}\n")).unwrap();
    git(&root, &["add", "."]);
    git(
        &root,
        &["-c", "core.hooksPath=/dev/null", "commit", "-qm", "pin"],
    );
    start(&root, "pin");
    commit(&root, "product.txt", "product");
    let product = git(&root, &["rev-parse", "HEAD"]);
    let pinned = |revision: &str| {
        fs::write(root.join("stable.txt"), format!("{revision}\n")).unwrap();
        committed(&root, "pin")
    };
    let forward = pinned(&product);
    assert_eq!(forward.code, Some(0), "{}", forward.output());
    let backward = pinned(&first);
    assert_eq!(backward.code, Some(1), "{}", backward.output());
    assert!(
        backward.stderr.contains("stable.txt"),
        "{}",
        backward.stderr
    );
}
