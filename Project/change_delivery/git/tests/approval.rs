// DECISION: D213
use super::{committed, installed, read, start};
use crate::shared::testing::{configure, git, invoke};
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
    let path = root.join(".agentrig").join(format!("{module}.yaml"));
    let mut value: Value = serde_yaml_ng::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    edit(&mut value);
    fs::write(path, serde_yaml_ng::to_string(&value).unwrap()).unwrap();
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

const BASELINE: &str = ".agentrig/lint-baseline.yaml";

fn wide(root: &Path, lines: usize) {
    let text = format!("{}\n", "x".repeat(130)).repeat(lines);
    fs::write(root.join("README.md"), text).unwrap();
}

fn recorded(root: &Path, expected: &str) {
    let run = invoke(root, &["lint-baseline"]);
    assert_eq!(run.code, Some(0), "{}", run.output());
    let baseline = read(root, BASELINE);
    assert!(baseline.contains(expected), "{baseline}");
}

fn refused(root: &Path, message: &str) {
    let run = committed(root, message);
    assert_ne!(run.code, Some(0), "{}", run.output());
    assert!(run.stderr.contains(BASELINE), "{}", run.stderr);
    git(root, &["reset", "-q", "--hard"]);
}

// DECISION: D214
#[test]
fn a_lint_baseline_commits_without_approval_only_when_it_accepts_no_new_error() {
    let (_temp, root) = installed();
    start(&root, "baseline");
    wide(&root, 2);
    recorded(&root, "README.md: 2");
    let introduced = committed(&root, "introduced");
    assert_eq!(introduced.code, Some(1), "{}", introduced.output());
    let approved = invoke(&root, &["approve"]);
    assert_eq!(approved.code, Some(0), "{}", approved.output());
    let accepted = committed(&root, "approved");
    assert_eq!(accepted.code, Some(0), "{}", accepted.output());
    let raised = read(&root, BASELINE).replace("README.md: 2", "README.md: 3");
    fs::write(root.join(BASELINE), raised).unwrap();
    refused(&root, "raised");
    fs::write(root.join(BASELINE), "{").unwrap();
    refused(&root, "corrupt");
    wide(&root, 1);
    recorded(&root, "README.md: 1");
    let lowered = committed(&root, "lowered");
    assert_eq!(lowered.code, Some(0), "{}", lowered.output());
}
