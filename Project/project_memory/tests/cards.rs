// DECISION: D242
use super::consumer::{accepts, card, consumer, edit, rejects};
use crate::shared::testing::{invoke, write};
use std::path::Path;

const CHECK: &[&str] = &["memory-check"];
const MISPLACED: [(&[&str], &str); 4] = [
    (
        &["notes/Backlog/RIG-207.md", "notes/Plan/RIG-207.md"],
        "RIG-207 lies in both notes/Backlog/RIG-207.md and notes/Plan/RIG-207.md",
    ),
    (
        &["notes/Plan/207.md"],
        "notes/Plan/207.md: a card is named RIG-207.md or RIG-207/",
    ),
    (
        &["notes/Plan/RIG-207/assets/plot.svg"],
        "notes/Plan/RIG-207: a card directory holds its card RIG-207.md",
    ),
    (
        &[
            "notes/Plan/RIG-207/RIG-207.md",
            "notes/Plan/RIG-207/notes.txt",
        ],
        "notes/Plan/RIG-207/notes.txt: a card directory holds only RIG-207.md, assets/, results/ and scenarios.md",
    ),
];
const TEXT: [(&str, &str, &str); 4] = [
    ("# RIG-1 —", "# RIG-2 —", "the first line of a card is"),
    (" Explain slow checks", "", "the first line of a card is"),
    ("## Proposal", "## Unexpected", "sections must be"),
    ("Show which check takes the time.", "", "empty section"),
];
const REGISTRIES: [&str; 4] = [
    "notes/Archive/Plan.md",
    "notes/Archive/Plan/001.md",
    "notes/Archive/Backlog.md",
    "notes/Archive/Backlog/001.md",
];

fn place(root: &Path, paths: &[&str]) {
    for path in paths {
        let card_file = path.ends_with(".md");
        if card_file {
            card(root, path, "");
        } else {
            write(root, path, "Material.\n");
        }
    }
}

#[test]
fn initialized_memory_accepts_a_card() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let initialized = invoke(&root, &["init", "--key", "RIG", "--skills", "guides"]);
    assert_eq!(initialized.code, Some(0), "{}", initialized.output());
    assert!(root.join("guides/edit-backlog/SKILL.md").is_file());
    accepts(&root);
    card(&root, "Ledger/Backlog/RIG-1.md", "");
    accepts(&root);
}

#[test]
fn misplaced_cards_are_refused_by_name() {
    for (paths, diagnostic) in MISPLACED {
        let (_temp, root) = consumer();
        place(&root, paths);
        rejects(&root, CHECK, diagnostic);
    }
}

#[test]
fn a_card_directory_holds_its_card_and_materials() {
    let (_temp, root) = consumer();
    card(&root, "notes/Plan/RIG-207/RIG-207.md", "");
    for material in ["assets/plot.svg", "results/delivery.md", "scenarios.md"] {
        let path = format!("notes/Plan/RIG-207/{material}");
        write(&root, &path, "Material.\n");
    }
    accepts(&root);
}

#[test]
fn card_text_is_checked() {
    for (before, after, diagnostic) in TEXT {
        let (_temp, root) = consumer();
        let path = card(&root, "notes/Backlog/RIG-1.md", "");
        edit(&path, before, after);
        rejects(&root, CHECK, diagnostic);
    }
}

#[test]
fn the_registries_before_cards_stay_in_the_archive() {
    let (_temp, root) = consumer();
    for registry in REGISTRIES {
        write(&root, registry, "Written before cards.\n");
    }
    card(&root, "notes/Archive/RIG-2.md", "");
    accepts(&root);
}

#[test]
fn cards_need_the_queue_key() {
    let (_temp, root) = consumer();
    edit(&root.join(".agentrig/project.yaml"), "key: RIG\n", "");
    accepts(&root);
    card(&root, "notes/Plan/RIG-1.md", "");
    rejects(&root, CHECK, ".agentrig/project.yaml names none");
}
