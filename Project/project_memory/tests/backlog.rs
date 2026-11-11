// DECISION: D050
use super::consumer::{accepts, consumer, edit, rejects};
use crate::shared::testing::invoke;
use std::{fs, path::Path};

const HEADER: &str = "# Backlog\n\n| ID | Idea | Expected benefit |\n| --- | --- | --- |\n";
const ROW: &str = "| [B001](Backlog/001.md) | Explain slow checks | Locate avoidable delays |\n";
const CARD: &str = "# B001 — Explain slow checks\n\n## Context\n\nA check appeared slow during unrelated feature work.\n\n## Proposal\n\nShow which check consumes the time.\n\n## Expected benefit\n\nHelp users identify avoidable delays; benefit is unverified.\n";
const CHECK: &[&str] = &["memory-check"];
const BROKEN: [(&str, &str, &str, &str); 5] = [
    (
        "Backlog.md",
        "avoidable delays |\n",
        "avoidable delays |\n| [B001](Backlog/001.md) | Repeated idea | Same identity |\n",
        "duplicate ID",
    ),
    (
        "Backlog.md",
        "Backlog/001.md",
        "Backlog/002.md",
        "detail path",
    ),
    ("Backlog.md", "B001", "P001", "invalid or duplicate ID"),
    (
        "Backlog/001.md",
        "## Proposal",
        "## Unexpected",
        "sections must be",
    ),
    (
        "Backlog/001.md",
        "Show which check consumes the time.",
        "",
        "empty section",
    ),
];
const LOST: [(&str, &str); 3] = [
    ("card", "missing or escaping memory link"),
    ("row", "unindexed detail"),
    ("index", "unindexed detail"),
];

fn write_idea(notes: &Path) {
    let index = notes.join("Backlog.md");
    let indexed = index.is_file();
    let current = if indexed {
        fs::read_to_string(&index).unwrap()
    } else {
        HEADER.to_owned()
    };
    fs::write(&index, format!("{current}{ROW}")).unwrap();
    fs::create_dir_all(notes.join("Backlog")).unwrap();
    fs::write(notes.join("Backlog/001.md"), CARD).unwrap();
}

fn lose(notes: &Path, part: &str) {
    let index = notes.join("Backlog.md");
    match part {
        "card" => fs::remove_file(notes.join("Backlog/001.md")).unwrap(),
        "row" => edit(&index, ROW, ""),
        _ => fs::remove_file(&index).unwrap(),
    }
}

#[test]
fn initialized_backlog_accepts_an_idea() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let initialized = invoke(&root, &["init", "--memory", "ideas", "--skills", "guides"]);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    assert!(root.join("guides/edit-backlog/SKILL.md").is_file());
    let notes = root.join("ideas");
    assert!(notes.join("Backlog.md").is_file());
    accepts(&root);
    write_idea(&notes);
    accepts(&root);
}

#[test]
fn backlog_rejects_broken_records() {
    let (_temp, root) = consumer();
    let notes = root.join("notes");
    write_idea(&notes);
    accepts(&root);
    for (relative, old, new, diagnostic) in BROKEN {
        let path = notes.join(relative);
        let original = fs::read_to_string(&path).unwrap();
        edit(&path, old, new);
        rejects(&root, CHECK, diagnostic);
        fs::write(&path, original).unwrap();
    }
    accepts(&root);
}

#[test]
fn archived_backlog_is_checked_and_keeps_ids() {
    let (_temp, root) = consumer();
    let notes = root.join("notes");
    write_idea(&notes);
    let archive = notes.join("Archive");
    fs::create_dir(&archive).unwrap();
    fs::rename(notes.join("Backlog.md"), archive.join("Backlog.md")).unwrap();
    fs::rename(notes.join("Backlog"), archive.join("Backlog")).unwrap();
    accepts(&root);
    let card = archive.join("Backlog/001.md");
    edit(&card, "## Proposal", "## Unexpected");
    rejects(&root, CHECK, "sections must be");
    fs::write(&card, CARD).unwrap();
    write_idea(&notes);
    rejects(&root, CHECK, "duplicate ID across Backlog and Archive");
}

#[test]
fn backlog_reports_lost_links() {
    for (part, diagnostic) in LOST {
        let (_temp, root) = consumer();
        let notes = root.join("notes");
        write_idea(&notes);
        lose(&notes, part);
        rejects(&root, CHECK, diagnostic);
    }
}
