// DECISION: D012
// DECISION: D031
use super::consumer::{accepts, consumer, edit, rejects};
use crate::shared::testing::snapshot;
use std::{
    fs,
    path::{Path, PathBuf},
};

type Row = (u32, &'static str, &'static str);

const HEADER: &str = "# Plan\n\n| ID | Status | Depends on | Feature | User capability |\n| --- | --- | --- | --- | --- |\n";
const PAUSED: &str = "\n## Delivery\n\nBlocked by a missing prior outcome. Retained branch: feature/example. Resume after prerequisites are delivered.\n";
const COMPLETE: &str = "\n## Delivery\n\nAcceptance verified by the feature test; passed.\n";
const CHECK: &[&str] = &["memory-check"];
const CONTRACT: [(&[Row], Option<&str>); 18] = [
    (&[], None),
    (&[(1, "pending", "-")], None),
    (&[(1, "active", "-")], None),
    (&[(1, "paused", "-")], None),
    (&[(1, "complete", "-")], None),
    (&[(1, "complete", "-"), (2, "active", "P001")], None),
    (&[(2, "active", "-"), (1, "paused", "P002")], None),
    (&[(1, "active", "-"), (2, "pending", "P001")], None),
    (
        &[(1, "active", "-"), (2, "active", "-")],
        Some("multiple active"),
    ),
    (
        &[(1, "pending", "-"), (1, "pending", "-")],
        Some("invalid or duplicate ID"),
    ),
    (&[(1, "pending", "P999")], Some("unknown dependency")),
    (&[(1, "pending", "P001")], Some("cycle")),
    (
        &[(1, "paused", "P002"), (2, "pending", "P001")],
        Some("cycle"),
    ),
    (
        &[(1, "pending", "-"), (2, "active", "P001")],
        Some("prerequisite"),
    ),
    (
        &[(1, "paused", "-"), (2, "complete", "P001")],
        Some("prerequisite"),
    ),
    (
        &[(1, "complete", "-"), (2, "pending", "P001, P001")],
        Some("duplicate dependencies"),
    ),
    (&[(1, "unknown", "-")], Some("invalid")),
    (&[(1, "pending", "P1")], Some("unknown dependency")),
];
const SECTIONS_FINDING: &str = "sections must be";
const SECTIONS: [(&str, &str); 4] = [
    ("## Feature", "## Implementation"),
    (
        "## Acceptance\n\nObservable acceptance.",
        "## Acceptance\n\n",
    ),
    ("## User capability", "## Feature"),
    (
        "## Acceptance",
        "## Arbitrary\n\nUnexpected.\n\n## Acceptance",
    ),
];
const ROWS: [(&str, &str, &str); 5] = [
    ("(Plan/001.md)", "(Plan/002.md)", "detail path"),
    (" | Product outcome |", " |   |", "empty table cell"),
    (" | Observable capability |", " |   |", "empty table cell"),
    (" | pending | - |", " | pending |", "expected 5 columns"),
    ("Depends on", "Dependencies", "table header"),
];
const LIFECYCLE: [&[Row]; 7] = [
    &[(1, "active", "-")],
    &[(1, "active", "-"), (2, "pending", "P001")],
    &[
        (3, "active", "-"),
        (1, "paused", "P003"),
        (2, "pending", "P001"),
    ],
    &[
        (3, "complete", "-"),
        (1, "paused", "P003"),
        (2, "pending", "P001"),
    ],
    &[
        (3, "complete", "-"),
        (1, "active", "P003"),
        (2, "pending", "P001"),
    ],
    &[
        (3, "complete", "-"),
        (1, "complete", "P003"),
        (2, "active", "P001"),
    ],
    &[
        (3, "complete", "-"),
        (1, "complete", "P003"),
        (2, "complete", "P001"),
    ],
];
const ARCHIVED: [(&str, &str, &str, &str); 3] = [
    (
        "Plan.md",
        "| complete |",
        "| active |",
        "Archive requires completed",
    ),
    (
        "Plan/001.md",
        "## Acceptance",
        "## Unexpected",
        "sections must be",
    ),
    (
        "Plan.md",
        "| complete | - |",
        "| complete | P002 |",
        "prerequisite",
    ),
];
const UNARCHIVED: [(&str, &str); 2] = [
    ("Plan.md", "unindexed detail"),
    ("Plan/001.md", "missing or escaping memory link"),
];

fn delivery(status: &str) -> &'static str {
    match status {
        "paused" => PAUSED,
        "complete" => COMPLETE,
        _ => "",
    }
}

fn write_plan(root: &Path, rows: &[Row]) {
    let notes = root.join("notes");
    fs::create_dir_all(notes.join("Plan")).unwrap();
    let mut table = String::from(HEADER);
    for (identity, status, dependencies) in rows {
        table.push_str(&format!(
            "| [P{identity:03}](Plan/{identity:03}.md) | {status} | {dependencies} | Product outcome | Observable capability |\n"
        ));
        let detail = format!(
            "# P{identity:03}\n\n## Feature\n\nProduct result.\n\n## User capability\n\nUser capability.\n\n## Acceptance\n\nObservable acceptance.\n{}",
            delivery(status)
        );
        fs::write(notes.join(format!("Plan/{identity:03}.md")), detail).unwrap();
    }
    fs::write(notes.join("Plan.md"), table).unwrap();
}

fn archive_plan(root: &Path) -> PathBuf {
    let notes = root.join("notes");
    let archive = notes.join("Archive");
    fs::create_dir(&archive).unwrap();
    fs::rename(notes.join("Plan.md"), archive.join("Plan.md")).unwrap();
    fs::rename(notes.join("Plan"), archive.join("Plan")).unwrap();
    archive
}

fn archived(current: &[Row]) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let (temp, root) = consumer();
    write_plan(&root, &[(1, "complete", "-")]);
    let archive = archive_plan(&root);
    write_plan(&root, current);
    (temp, root, archive)
}

// INVARIANT: I009
#[test]
fn plan_delivery_contract() {
    for (rows, expected) in CONTRACT {
        let (_temp, root) = consumer();
        write_plan(&root, rows);
        match expected {
            None => accepts(&root),
            Some(diagnostic) => rejects(&root, CHECK, diagnostic),
        }
    }
}

#[test]
fn invalid_detail_sections_fail() {
    for (old, new) in SECTIONS {
        let (_temp, root) = consumer();
        write_plan(&root, &[(1, "pending", "-")]);
        edit(&root.join("notes/Plan/001.md"), old, new);
        rejects(&root, CHECK, SECTIONS_FINDING);
    }
}

#[test]
fn delivery_context_required() {
    for status in ["paused", "complete"] {
        let (_temp, root) = consumer();
        write_plan(&root, &[(1, status, "-")]);
        let path = root.join("notes/Plan/001.md");
        let content = fs::read_to_string(&path).unwrap();
        let (context, _) = content.split_once("\n## Delivery").unwrap();
        fs::write(&path, context).unwrap();
        rejects(&root, CHECK, "requires Delivery");
        fs::write(&path, format!("{context}\n## Delivery\n\n")).unwrap();
        rejects(&root, CHECK, SECTIONS_FINDING);
    }
}

#[test]
fn malformed_rows_are_not_silently_skipped() {
    for (old, new, diagnostic) in ROWS {
        let (_temp, root) = consumer();
        write_plan(&root, &[(1, "pending", "-")]);
        edit(&root.join("notes/Plan.md"), old, new);
        rejects(&root, CHECK, diagnostic);
    }
}

#[test]
fn missing_detail_fails() {
    let (_temp, root) = consumer();
    write_plan(&root, &[(1, "active", "-")]);
    fs::remove_file(root.join("notes/Plan/001.md")).unwrap();
    rejects(&root, CHECK, "missing or escaping memory link");
}

#[test]
fn blocker_followup_and_resume_lifecycle() {
    let (_temp, root) = consumer();
    for rows in LIFECYCLE {
        write_plan(&root, rows);
        accepts(&root);
    }
}

#[test]
fn current_plan_can_depend_on_archived_delivery() {
    let (_temp, root) = consumer();
    write_plan(&root, &[(1, "complete", "-"), (2, "complete", "P001")]);
    let archive = archive_plan(&root);
    let before = snapshot(&archive);
    write_plan(&root, &[(3, "active", "P002")]);
    accepts(&root);
    assert_eq!(snapshot(&archive), before);
}

#[test]
fn archived_plan_rejects_invalid_records() {
    for (relative, old, new, diagnostic) in ARCHIVED {
        let (_temp, root, archive) = archived(&[(2, "active", "P001")]);
        edit(&archive.join(relative), old, new);
        rejects(&root, CHECK, diagnostic);
    }
}

#[test]
fn archive_reports_missing_index_or_card() {
    for (missing, diagnostic) in UNARCHIVED {
        let (_temp, root, archive) = archived(&[]);
        fs::remove_file(archive.join(missing)).unwrap();
        rejects(&root, CHECK, diagnostic);
    }
}

#[test]
fn plan_cannot_reuse_an_archived_id() {
    let (_temp, root, _archive) = archived(&[(1, "active", "-")]);
    rejects(&root, CHECK, "duplicate ID across Plan and Archive");
}
