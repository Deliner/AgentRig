// DECISION: D003
// DECISION: D011
// DECISION: D014
use super::{DECISION, STATE, commit, committed, installed, read, start};
use crate::project_memory::state_template;
use crate::shared::testing::{describe_state, git, git_run, hermetic, invoke, run, runtime};
use std::{fs, path::Path};

const PRIOR_DECISION: &str = "# D001\n\n## Context\n\nText.\n\n## Chosen\n\nText.\n\n## Rejected\n\nText.\n\n## Rationale\n\nText.\n\n## Consequences\n\nText.\n";

// DECISION: D236
#[test]
fn after_init_the_commit_gate_takes_a_new_decision_only_in_three_sections() {
    let (_temp, root) = installed();
    start(&root, "decision");
    let row = "| [D001](Decisions/001.md) | Choice | [settings](../.agentrig/git.yaml) |\n";
    let index = read(&root, "Ledger/Decisions.md") + row;
    fs::write(root.join("Ledger/Decisions.md"), index).unwrap();
    fs::create_dir_all(root.join("Ledger/Decisions")).unwrap();
    fs::write(root.join("Ledger/Decisions/001.md"), PRIOR_DECISION).unwrap();
    let refused = committed(&root, "prior sections");
    assert_ne!(refused.code, Some(0), "{}", refused.output());
    let sections = ["Decision", "Reasons", "Alternatives"];
    let named = format!("Decisions/001.md: sections must be {sections:?}");
    assert!(refused.output().contains(&named), "{}", refused.output());
    fs::write(root.join("Ledger/Decisions/001.md"), DECISION).unwrap();
    let accepted = committed(&root, "three sections");
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
}

// DECISION: D204
const FROM_FEATURE: [&str; 4] = [
    "branch -f trunk HEAD",
    "checkout -B trunk",
    "switch -C trunk",
    "branch -D trunk",
];
const FROM_BASE: [&str; 2] = ["rebase task/moved", "cherry-pick task/moved"];

// INVARIANT: I006
#[test]
fn staged_commit_preserves_unstaged_work() {
    let (_temp, root) = installed();
    start(&root, "atomic");
    let before = git(&root, &["rev-parse", "HEAD"]);
    fs::write(root.join("product.txt"), "staged").unwrap();
    describe_state(&root, "A test VAC.");
    git(&root, &["add", "product.txt", STATE]);
    fs::write(root.join("product.txt"), "later unstaged work").unwrap();
    fs::write(root.join("unrelated.txt"), "user work").unwrap();
    let mut blocked = hermetic("git");
    blocked
        .args(["commit", "-qm", "blocked"])
        .current_dir(&root)
        .env("BLOCK_DELIVERY", "1");
    let blocked = run(&mut blocked, "");
    assert_ne!(blocked.code, Some(0), "{}", blocked.output());
    assert_eq!(git(&root, &["rev-parse", "HEAD"]), before);
    git(&root, &["commit", "-qm", "verified index"]);
    assert_eq!(git(&root, &["show", "HEAD:product.txt"]), "staged");
    assert_eq!(read(&root, "product.txt"), "later unstaged work");
    let committed = git(&root, &["ls-tree", "--name-only", "HEAD"]);
    assert!(!committed.lines().any(|name| name == "unrelated.txt"));
    assert_eq!(read(&root, "unrelated.txt"), "user work");
}

// DECISION: D191
#[test]
fn git_hooks_guard_operations_started_in_a_subdirectory() {
    let (_temp, root) = installed();
    start(&root, "nested");
    let nested = root.join("nested");
    fs::create_dir(&nested).unwrap();
    fs::write(nested.join("product.txt"), "product").unwrap();
    describe_state(&root, "A test VAC.");
    git(&nested, &["add", "-A", ".."]);
    let mut blocked = hermetic("git");
    blocked
        .args(["commit", "-qm", "blocked"])
        .current_dir(&nested)
        .env("BLOCK_DELIVERY", "1");
    let blocked = run(&mut blocked, "");
    assert_ne!(blocked.code, Some(0), "{}", blocked.output());
    git(&nested, &["commit", "-qm", "nested"]);
    assert_eq!(read(&root, STATE), state_template());
    git(&nested, &["branch", "task/merged", "trunk"]);
    let deleted = git_run(&nested, &["branch", "-D", "task/merged"]);
    assert_ne!(deleted.code, Some(0), "{}", deleted.output());
    assert!(deleted.stderr.contains("retained"), "{}", deleted.stderr);
}

// DECISION: D191
#[test]
fn feature_commits_carry_a_described_state() {
    let (_temp, root) = installed();
    start(&root, "described");
    fs::write(root.join("product.txt"), "product").unwrap();
    git(&root, &["add", "product.txt"]);
    let empty = git_run(&root, &["commit", "-qm", "empty State"]);
    assert_eq!(empty.code, Some(1), "{}", empty.output());
    let skill = root.join(".agentrig/skills/edit-state/SKILL.md");
    let named = empty.stderr.contains(skill.to_str().unwrap());
    assert!(named, "{}", empty.stderr);
    describe_state(&root, "A test VAC.");
    git(&root, &["add", STATE]);
    git(&root, &["commit", "-qm", "described"]);
    fs::write(root.join(STATE), state_template()).unwrap();
    git(&root, &["add", STATE]);
    git(&root, &["commit", "-qm", "reset only"]);
    fs::write(root.join("later.txt"), "later").unwrap();
    git(&root, &["add", "later.txt"]);
    git(&root, &["rm", "-q", "--cached", STATE]);
    let removed = git_run(&root, &["commit", "-qm", "removed State"]);
    assert_eq!(removed.code, Some(1), "{}", removed.output());
    let failed = removed.stderr.contains("git failed") && removed.stderr.contains(STATE);
    assert!(failed, "{}", removed.stderr);
}

// INVARIANT: I021
#[test]
fn reference_guard_rejects_deletion_when_the_tip_cannot_be_resolved() {
    let (_temp, root) = installed();
    git(
        &root,
        &["-c", "core.hooksPath=/dev/null", "branch", "task/merged"],
    );
    let unknown = "0".repeat(40);
    let delete = |branch: &str| {
        let update = format!("{unknown} {unknown} refs/heads/task/{branch}\n");
        run(
            &mut runtime(&root, &["guard-reference", "prepared"]),
            &update,
        )
    };
    let merged = delete("merged");
    assert_eq!(merged.code, Some(1), "{}", merged.output());
    assert!(merged.stderr.contains("retained"), "{}", merged.stderr);
    let missing = delete("missing");
    assert_eq!(missing.code, Some(2), "{}", missing.output());
    assert!(!missing.stderr.contains("retained"), "{}", missing.stderr);
}

fn refused(root: &Path, words: &str, base: &str) {
    let args: Vec<_> = ["run", "write", "--", "git"]
        .into_iter()
        .chain(words.split(' '))
        .collect();
    let attempt = invoke(root, &args);
    assert_ne!(attempt.code, Some(0), "{words}: {}", attempt.output());
    let named = attempt.stderr.contains("trunk changes only through");
    assert!(named, "{words}: {}", attempt.output());
    assert_eq!(git(root, &["rev-parse", "trunk"]), base, "{words}");
}

// DECISION: D204
#[test]
fn write_commands_cannot_change_the_base() {
    let (_temp, root) = installed();
    let base = git(&root, &["rev-parse", "trunk"]);
    start(&root, "moved");
    commit(&root, "feature.txt", "feature");
    for words in FROM_FEATURE {
        refused(&root, words, &base);
    }
    for words in FROM_BASE {
        git(&root, &["switch", "-qf", "trunk"]);
        refused(&root, words, &base);
        let (operation, _) = words.split_once(' ').unwrap();
        git_run(&root, &[operation, "--abort"]);
    }
    git(&root, &["switch", "-qf", "trunk"]);
    fs::write(root.join(".git/MERGE_HEAD"), "").unwrap();
    fs::write(root.join("merged.txt"), "merged").unwrap();
    git(&root, &["add", "merged.txt"]);
    refused(&root, "commit -qm merged", &base);
}
