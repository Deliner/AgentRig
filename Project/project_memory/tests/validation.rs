// DECISION: D020
// DECISION: D012
// DECISION: D009
// DECISION: D008
// DECISION: D007
// DECISION: D002
use super::consumer::{
    accepts, append, committed, consumer, decision, detail, edit, record, rejects,
};
use crate::shared::testing::{commit_identity, git, invoke, repository, snapshot};
use serde_json::Value;
use std::fs;

const EXAMPLES: [(&str, &str, &str); 3] = [
    ("rs", "fn example() {}", "//"),
    ("py", "value = 1", "#"),
    ("sh", "echo example", "#"),
];
const PRIOR: [&str; 5] = ["Context", "Chosen", "Rejected", "Rationale", "Consequences"];
const CURRENT: [&str; 3] = ["Decision", "Reasons", "Alternatives"];

fn layout(number: &str) -> String {
    format!("Decisions/{number}.md: sections must be {CURRENT:?}")
}

// DECISION: D236
#[test]
fn a_new_decision_has_three_sections_while_a_committed_one_keeps_the_prior_five() {
    let (_temp, root) = consumer();
    record(&root, "src/lib.rs");
    let source = "// DECISION: D001\nfn example() {}";
    fs::write(root.join("src/lib.rs"), source).unwrap();
    fs::write(root.join("notes/Decisions/001.md"), detail("D001", &PRIOR)).unwrap();
    rejects(&root, &["memory-check"], &layout("001"));
    git(&root, &["init", "-q"]);
    commit_identity(&root);
    git(&root, &["add", "."]);
    git(&root, &["commit", "-qm", "prior"]);
    accepts(&root);
    let row = "| [D002](Decisions/002.md) | Second | [index](Decisions.md) |\n";
    append(&root.join("notes/Decisions.md"), row);
    let second = root.join("notes/Decisions/002.md");
    fs::write(&second, detail("D002", &PRIOR)).unwrap();
    git(&root, &["add", "."]);
    rejects(&root, &["memory-check"], &layout("002"));
    rejects(&root, &["check", "--staged"], &layout("002"));
    fs::write(&second, detail("D002", &CURRENT)).unwrap();
    git(&root, &["add", "."]);
    accepts(&root);
    let staged = invoke(&root, &["check", "--staged"]);
    assert_eq!(staged.code, Some(0), "{}", staged.output());
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
fn malformed_linked_sources_fail_at_the_first_link() {
    let (_temp, root) = consumer();
    record(&root, "src/zeta.rs");
    let linked = "[code](../src/zeta.rs), [parse](../src/lib.rs)";
    edit(
        &root.join("notes/Decisions.md"),
        "[code](../src/zeta.rs)",
        linked,
    );
    for name in ["src/zeta.rs", "src/lib.rs"] {
        fs::write(root.join(name), "fn broken( {\n").unwrap();
    }
    let first = format!("malformed source {}", root.join("src/zeta.rs").display());
    rejects(&root, &["memory-check"], &first);
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

// DECISION: D216
#[test]
fn a_decision_marker_counts_only_in_a_file_its_decision_links() {
    let (_temp, root) = consumer();
    record(&root, "src/lib.rs");
    fs::write(
        root.join("src/lib.rs"),
        "// DECISION: D001\nfn example() {}",
    )
    .unwrap();
    let lifted = "fn read() -> u8 {\n    // DECISION: D001\n    \"1\".parse().unwrap()\n}\n";
    fs::write(root.join("src/other.rs"), lifted).unwrap();
    rejects(&root, &["memory-check"], "D001 in src/other.rs");
    let linked = "[code](../src/lib.rs), [parse](../src/other.rs)";
    edit(
        &root.join("notes/Decisions.md"),
        "[code](../src/lib.rs)",
        linked,
    );
    accepts(&root);
}

// DECISION: D219
#[test]
fn a_decision_lists_each_exception_and_each_listed_exception_exists() {
    let (_temp, root) = consumer();
    record(&root, "src/lib.rs");
    let source = root.join("src/lib.rs");
    let excepted =
        "fn read() -> u8 {\n    // DECISION: D001 panic-path\n    \"1\".parse().unwrap()\n}\n";
    fs::write(&source, excepted).unwrap();
    let unlisted = "remove the exception: D001 panic-path in src/lib.rs";
    rejects(&root, &["memory-check"], unlisted);
    let listed = "[exception panic-path](../src/lib.rs)";
    edit(
        &root.join("notes/Decisions.md"),
        "[code](../src/lib.rs)",
        listed,
    );
    accepts(&root);
    fs::write(&source, "// DECISION: D001\nfn read() -> u8 {\n    1\n}\n").unwrap();
    let absent = "restore the exception: D001 panic-path in src/lib.rs";
    rejects(&root, &["memory-check"], absent);
}

// DECISION: D219
#[test]
fn an_exception_counts_in_a_trailing_comment_and_not_in_a_string() {
    let (_temp, root) = consumer();
    record(&root, "src/lib.rs");
    let source = root.join("src/lib.rs");
    let quoted = "// DECISION: D001\nconst TEXT: &str = \"// DECISION: D001 panic-path\";\n";
    fs::write(&source, quoted).unwrap();
    accepts(&root);
    let trailing = "// DECISION: D001\nfn read() -> u8 {\n    \"1\".parse().unwrap() // DECISION: D001 panic-path\n}\n";
    fs::write(&source, trailing).unwrap();
    let unlisted = "remove the exception: D001 panic-path in src/lib.rs";
    rejects(&root, &["memory-check"], unlisted);
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
    assert!(repository().join("Ledger/Decisions.md").is_file());
    let (_temp, root) = consumer();
    let detail = root.join("notes/Decisions/001.md");
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
