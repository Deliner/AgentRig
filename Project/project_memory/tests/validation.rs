// DECISION: D020
// DECISION: D012
// DECISION: D009
// DECISION: D008
// DECISION: D007
// DECISION: D002
use super::consumer::{accepts, append, committed, consumer, decision, record};
use crate::shared::testing::{git, invoke, repository, snapshot};
use serde_json::Value;
use std::{fs, path::Path};

const EXAMPLES: [(&str, &str, &str); 3] = [
    ("rs", "fn example() {}", "//"),
    ("py", "value = 1", "#"),
    ("sh", "echo example", "#"),
];
const PLAN_HEADINGS: [&str; 3] = ["Feature", "User capability", "Acceptance"];

fn rejects(root: &Path, args: &[&str], diagnostic: &str) {
    let result = invoke(root, args);
    assert_eq!(result.code, Some(2), "{}", result.output());
    assert!(result.stderr.contains(diagnostic), "{}", result.stderr);
}

fn edit(path: &Path, before: &str, after: &str) {
    let text = fs::read_to_string(path).unwrap();
    fs::write(path, text.replace(before, after)).unwrap();
}

#[test]
fn resume_reports_a_stale_snapshot_without_writing() {
    let (_temp, root) = consumer();
    accepts(&root);
    let workspace = "## Workspace\n\nBranch: `missing`";
    edit(
        &root.join("notes/State.md"),
        "## Workspace\n\nNone.",
        workspace,
    );
    let before = snapshot(&root);
    let result = invoke(&root, &["resume"]);
    assert_eq!(result.code, Some(0), "{}", result.output());
    let report: Value = serde_json::from_str(&result.stdout).unwrap();
    assert_eq!(report["snapshot"], "stale");
    assert_eq!(snapshot(&root), before);
}

#[test]
fn rust_decision_markers_are_comments() {
    let (_temp, root) = consumer();
    record(&root, "src/lib.rs");
    let source = root.join("src/lib.rs");
    fs::write(&source, "const TEXT: &str = \"// DECISION: D001\";").unwrap();
    rejects(&root, &["memory-check"], "marker absent");
    fs::write(&source, "// DECISION: D001\nfn example() {}").unwrap();
    accepts(&root);
    fs::remove_file(&source).unwrap();
    rejects(&root, &["memory-check"], "missing or escaping memory link");
}

#[test]
fn source_references_name_recorded_entries() {
    let (_temp, root) = consumer();
    let source = root.join("src/lib.py");
    for (kind, identifier) in [("DECISION", "D404"), ("INVARIANT", "I404")] {
        fs::write(&source, format!("# {kind}: {identifier}\nvalue = 1\n")).unwrap();
        let diagnostic = format!("src/lib.py: {identifier} is not a recorded");
        rejects(&root, &["memory-check"], &diagnostic);
    }
    fs::write(&source, "value = 1\n").unwrap();
    fs::write(root.join("src/lib.cpython-313.pyc"), b"\xf3\r\r\n").unwrap();
    accepts(&root);
}

#[test]
fn plan_dependency_cycle_is_rejected() {
    let (_temp, root) = consumer();
    let notes = root.join("notes");
    for (identity, dependency) in [("001", "002"), ("002", "001")] {
        let row = format!(
            "| [P{identity}](Plan/{identity}.md) | pending | P{dependency} | Result | Capability |\n"
        );
        append(&notes.join("Plan.md"), &row);
        let sections = PLAN_HEADINGS.map(|heading| format!("## {heading}\n\nText."));
        let detail = format!("# Feature\n\n{}", sections.join("\n\n"));
        fs::write(notes.join(format!("Plan/{identity}.md")), detail).unwrap();
    }
    rejects(&root, &["memory-check"], "dependency cycle");
}

// INVARIANT: I003
#[test]
fn committed_decisions_checked_against_staged_memory() {
    let (_temp, root) = committed();
    let index = root.join("notes/Decisions.md");
    let detail = root.join("notes/Decisions/001.md");
    let listed = fs::read_to_string(&index).unwrap();
    fs::write(&index, listed.replace("| Choice |", "| Changed |")).unwrap();
    let identity = "committed decision identity cannot change";
    rejects(&root, &["memory-check"], identity);
    fs::rename(root.join("src/lib.rs"), root.join("src/moved.rs")).unwrap();
    fs::write(&index, listed.replace("src/lib.rs", "src/moved.rs")).unwrap();
    accepts(&root);
    fs::rename(root.join("src/moved.rs"), root.join("src/lib.rs")).unwrap();
    fs::write(&index, &listed).unwrap();
    let recorded = fs::read_to_string(&detail).unwrap();
    fs::write(&detail, recorded.replace("Text.", "Changed.")).unwrap();
    git(&root, &["add", "notes/Decisions/001.md"]);
    fs::write(&detail, &recorded).unwrap();
    accepts(&root);
    let staged = "committed decision detail cannot change";
    rejects(&root, &["check", "--staged"], staged);
    fs::write(&index, listed.replace(&decision("src/lib.rs"), "")).unwrap();
    rejects(
        &root,
        &["memory-check"],
        "committed decision cannot be removed",
    );
}

// INVARIANT: I001
#[test]
fn decision_source_markers() {
    for (suffix, code, marker) in EXAMPLES {
        let (_temp, root) = consumer();
        let name = format!("src/example.{suffix}");
        record(&root, &name);
        let source = root.join(&name);
        fs::write(&source, code).unwrap();
        rejects(&root, &["memory-check"], "marker absent");
        fs::write(&source, format!("{marker} DECISION: D001\n{code}")).unwrap();
        accepts(&root);
    }
}

// INVARIANT: I008
#[test]
fn repository_layout_and_detail_contract() {
    assert!(repository().join("Project/README.md").is_file());
    assert!(repository().join("Ledger/Plan.md").is_file());
    let (_temp, root) = consumer();
    let detail = root.join("notes/Plan/001.md");
    fs::write(&detail, "# Unindexed").unwrap();
    rejects(&root, &["memory-check"], "unindexed detail");
    fs::remove_file(&detail).unwrap();
    accepts(&root);
}

#[test]
fn committed_decisions_are_preserved_in_worktree_and_gate() {
    let (_temp, root) = committed();
    let index = root.join("notes/Decisions.md");
    let detail = root.join("notes/Decisions/001.md");
    let listed = fs::read_to_string(&index).unwrap();
    let recorded = fs::read_to_string(&detail).unwrap();
    let changes = [
        (
            &index,
            listed.replace("| Choice |", "| Changed |"),
            "identity cannot change",
        ),
        (
            &detail,
            recorded.replace("Text.", "Changed."),
            "detail cannot change",
        ),
        (
            &index,
            listed.replace(&decision("src/lib.rs"), ""),
            "cannot be removed",
        ),
    ];
    for (path, content, diagnostic) in changes {
        let original = fs::read_to_string(path).unwrap();
        fs::write(path, content).unwrap();
        rejects(&root, &["memory-check"], diagnostic);
        rejects(&root, &["check", "--only", "memory"], diagnostic);
        fs::write(path, original).unwrap();
        accepts(&root);
    }
}

#[test]
fn committed_decisions_follow_a_relocated_memory() {
    let (_temp, root) = committed();
    edit(
        &root.join(".agentrig/project.yaml"),
        "memory: notes",
        "memory: relocated",
    );
    fs::rename(root.join("notes"), root.join("relocated")).unwrap();
    accepts(&root);
    edit(
        &root.join("relocated/Decisions.md"),
        "| Choice |",
        "| Changed |",
    );
    let identity = "committed decision identity cannot change";
    rejects(&root, &["memory-check"], identity);
}
