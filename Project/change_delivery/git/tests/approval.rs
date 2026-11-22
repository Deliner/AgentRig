// DECISION: D213
use super::{DECISION, committed, installed, read, start};
use crate::shared::testing::{configure, edit_yaml, git, invoke};
use serde_json::{Value, json};
use std::{fs, path::Path};

const CATALOG: &str = ".agentrig/commands.yaml";
const PROPOSAL: &str = ".cache/runtime/proposal";
const RECORD: &str = ".cache/runtime/approved";
const ADDED: &str = "extra:\n  purpose: Run an approved addition\n  argv: [sh, -c, 'true']\n";

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

// DECISION: D220
#[test]
fn a_new_lint_exception_commits_only_after_the_user_approves_its_line() {
    let (_temp, root) = installed();
    start(&root, "exception");
    let row = "| [D001](Decisions/001.md) | Choice | [exception panic-path](../probe.yaml) |\n";
    let index = read(&root, "Ledger/Decisions.md");
    fs::write(root.join("Ledger/Decisions.md"), format!("{index}{row}")).unwrap();
    fs::create_dir_all(root.join("Ledger/Decisions")).unwrap();
    fs::write(root.join("Ledger/Decisions/001.md"), DECISION).unwrap();
    let probe = root.join("probe.yaml");
    fs::write(&probe, "# DECISION: D001 panic-path\nkey: value\n").unwrap();
    let shown = "probe.yaml:1 D001 panic-path above key: value";
    let unapproved = committed(&root, "unapproved");
    assert_eq!(unapproved.code, Some(1), "{}", unapproved.output());
    assert!(unapproved.stderr.contains(shown), "{}", unapproved.stderr);
    let approved = invoke(&root, &["approve"]);
    assert_eq!(approved.code, Some(0), "{}", approved.output());
    assert!(approved.stdout.contains(shown), "{}", approved.stdout);
    let accepted = committed(&root, "approved");
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
    fs::write(&probe, "# DECISION: D001 panic-path\nkey: changed\n").unwrap();
    let edited = committed(&root, "edited");
    assert_eq!(edited.code, Some(1), "{}", edited.output());
    assert!(
        edited.stderr.contains("above key: changed"),
        "{}",
        edited.stderr
    );
    fs::write(&probe, "key: changed\n").unwrap();
    let unlisted = read(&root, "Ledger/Decisions.md").replace("exception panic-path", "file");
    fs::write(root.join("Ledger/Decisions.md"), unlisted).unwrap();
    let removed = committed(&root, "removed");
    assert_eq!(removed.code, Some(0), "{}", removed.output());
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

fn edited(root: &Path, module: &str, edit: impl FnOnce(&mut Value)) {
    edit_yaml(&root.join(".agentrig").join(format!("{module}.yaml")), edit);
}

fn probe(checks: &mut Value) -> &mut Value {
    let all = checks.as_array_mut().unwrap();
    all.iter_mut().find(|check| check["id"] == "probe").unwrap()
}

fn weakenings() -> [(&'static str, fn(&Path)); 6] {
    let checks = ".agentrig/checks.yaml";
    [
        (CATALOG, |root| {
            edited(root, "commands", |all| {
                all["probe"]["argv"] = json!(["sh", "--version"])
            });
        }),
        (checks, |root| {
            edited(root, "checks", |all| probe(all)["include"] = json!([]));
        }),
        (checks, |root| {
            edited(root, "checks", |all| probe(all)["warning"] = json!(true));
        }),
        (checks, |root| {
            edited(root, "checks", |all| probe(all)["affected"] = json!([]));
        }),
        ("differences.yaml", |root| {
            fs::write(root.join("differences.yaml"), "- reason: any\n").unwrap();
        }),
        (".agentrig/lint.yaml", |root| {
            let lint = read(root, ".agentrig/lint.yaml").replacen("error: 120", "error: 1000", 1);
            fs::write(root.join(".agentrig/lint.yaml"), lint).unwrap();
        }),
    ]
}

// DECISION: D214
#[test]
fn a_change_of_check_settings_commits_only_with_the_users_approval() {
    let (_temp, root) = installed();
    edited(&root, "commands", |all| {
        all["probe"]["accepts_args"] = json!(true)
    });
    edited(&root, "checks", |checks| {
        probe(checks)["affected"] = json!([{"include": ["**"], "targets": ["probe"]}]);
    });
    fs::write(root.join("differences.yaml"), "- reason: listed\n").unwrap();
    configure(&root, "hooks", "protected: [differences.yaml]\n");
    git(&root, &["add", "."]);
    let unhooked = ["-c", "core.hooksPath=/dev/null"];
    git(&root, &[&unhooked[..], &["commit", "-qm", "base"]].concat());
    start(&root, "settings");
    let examples = weakenings();
    for (path, weaken) in examples {
        weaken(&root);
        let refused = committed(&root, "weakened");
        assert_eq!(refused.code, Some(1), "{path}: {}", refused.output());
        assert!(refused.stderr.contains(path), "{path}: {}", refused.stderr);
        git(&root, &["reset", "-q", "--hard"]);
    }
    let (_, weaken) = examples[3];
    weaken(&root);
    let approved = invoke(&root, &["approve"]);
    assert_eq!(approved.code, Some(0), "{}", approved.output());
    let accepted = committed(&root, "approved");
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
}

fn rules(root: &Path, edit: impl FnOnce(&mut Vec<Value>)) {
    edited(root, "lint", |lint| {
        edit(lint["rules"].as_array_mut().unwrap())
    });
}

// DECISION: D226
#[test]
fn a_lint_configuration_commits_without_approval_only_when_it_adds_rules() {
    let (_temp, root) = installed();
    start(&root, "rules");
    rules(&root, |all| {
        let mut added = all
            .iter()
            .find(|rule| rule["id"] == "file-size")
            .unwrap()
            .clone();
        added["id"] = json!("added-file-size");
        all.insert(1, added);
    });
    let accepted = committed(&root, "added");
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
    let weakenings: [fn(&mut Vec<Value>); 3] = [
        |all| all.retain(|rule| rule["id"] != "file-size"),
        |all| all[0]["enabled"] = json!(false),
        |all| {
            let size = all.iter_mut().find(|rule| rule["id"] == "file-size");
            size.unwrap()["error"] = json!(100_000);
        },
    ];
    for weaken in weakenings {
        rules(&root, weaken);
        let refused = committed(&root, "weakened");
        assert_eq!(refused.code, Some(1), "{}", refused.output());
        for named in [".agentrig/lint.yaml", "has not approved"] {
            assert!(refused.stderr.contains(named), "{}", refused.stderr);
        }
        git(&root, &["reset", "-q", "--hard"]);
    }
}

const BASELINE: &str = ".agentrig/lint-baseline.yaml";

// DECISION: D235
#[test]
fn a_leftover_lint_baseline_needs_no_approval_and_accepts_no_error() {
    let (_temp, root) = installed();
    start(&root, "baseline");
    let accepting = "markdown-width:\n  README.md: 1\n";
    fs::write(root.join(BASELINE), accepting).unwrap();
    let added = committed(&root, "added");
    assert_eq!(added.code, Some(0), "{}", added.output());
    fs::write(root.join("README.md"), format!("{}\n", "x".repeat(130))).unwrap();
    let wide = committed(&root, "wide");
    assert_eq!(wide.code, Some(1), "{}", wide.output());
    assert!(
        wide.output().contains("markdown-width"),
        "{}",
        wide.output()
    );
    git(&root, &["reset", "-q", "--hard"]);
    fs::write(root.join(BASELINE), "markdown-width:\n  README.md: 2\n").unwrap();
    let raised = committed(&root, "raised");
    assert_eq!(raised.code, Some(0), "{}", raised.output());
    fs::remove_file(root.join(BASELINE)).unwrap();
    let removed = committed(&root, "removed");
    assert_eq!(removed.code, Some(0), "{}", removed.output());
}
