use super::consumer::{accepts, consumer, edit, rejects};
use crate::shared::testing::{configure, write};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

type Placed = (u32, &'static str);

const CHECK: &[&str] = &["memory-check"];
const NAMES: [&str; 3] = ["RIG-07.md", "RIG-7.txt", "RIG-x"];
const TITLE: &str = "the first line is # RIG-7 — and the card name";
const TEXTS: [(&str, &str, &str); 3] = [
    ("# RIG-7 — Card 7", "# RIG-8 — Card 7", TITLE),
    ("# RIG-7 — Card 7", "# RIG-7 — ", TITLE),
    ("## Proposal", "## Delivery", "sections must be"),
];
const PREAMBLE: &str = "at most one line, After: RIG-N";
const ORDERS: [(&[Placed], &str); 5] = [
    (
        &[(1, "After: RIG-9")],
        "After names RIG-9, a card in none of Backlog, Plan, Archive",
    ),
    (&[(1, "After: RIG-1")], "cycle"),
    (&[(1, "After: RIG-2"), (2, "After: RIG-1")], "cycle"),
    (&[(1, "After RIG-2"), (2, "")], PREAMBLE),
    (
        &[(1, "After: RIG-2\nAfter: RIG-3"), (2, ""), (3, "")],
        PREAMBLE,
    ),
];

fn queue() -> (TempDir, PathBuf) {
    let (temp, root) = consumer();
    configure(&root, "project", "key: RIG\n");
    (temp, root)
}

fn card(number: u32, after: &str) -> String {
    format!(
        "# RIG-{number} — Card {number}\n\n{after}\n\n## Context\n\nWhy.\n\n## Proposal\n\nWhat.\n\n## Expected benefit\n\nGain.\n"
    )
}

fn place(root: &Path, path: &str, text: &str) -> PathBuf {
    let relative = format!("notes/{path}");
    write(root, &relative, text);
    root.join(relative)
}

#[test]
fn a_card_lies_as_a_file_or_a_directory_with_materials_in_backlog_plan_or_archive() {
    let (_temp, root) = queue();
    place(&root, "Backlog/RIG-1.md", &card(1, ""));
    let ordered = card(2, "After: RIG-1, RIG-3");
    place(&root, "Plan/RIG-2/RIG-2.md", &ordered);
    place(&root, "Plan/RIG-2/assets/sketch.svg", "<svg/>");
    place(&root, "Plan/RIG-2/results/run.log", "passed");
    place(&root, "Plan/RIG-2/scenarios.md", "# Scenarios\n");
    place(&root, "Archive/RIG-3.md", &card(3, ""));
    accepts(&root);
}

#[test]
fn one_card_number_lies_in_one_place() {
    let (_temp, root) = queue();
    let backlog = place(&root, "Backlog/RIG-7.md", &card(7, ""));
    let plan = place(&root, "Plan/RIG-7.md", &card(7, ""));
    let both = format!("{} and {}", backlog.display(), plan.display());
    rejects(&root, CHECK, &both);
}

#[test]
fn a_card_directory_holds_its_card_and_only_its_materials() {
    let (_temp, root) = queue();
    place(&root, "Plan/RIG-7/assets/sketch.svg", "<svg/>");
    let directory = root.join("notes/Plan/RIG-7");
    let lacking = format!(
        "{}: a card directory holds its card RIG-7.md",
        directory.display()
    );
    rejects(&root, CHECK, &lacking);
    place(&root, "Plan/RIG-7/RIG-7.md", &card(7, ""));
    accepts(&root);
    let loose = place(&root, "Plan/RIG-7/notes.txt", "Loose note.");
    let foreign = format!(
        "{}: a card directory holds only RIG-7.md, assets, results, scenarios.md",
        loose.display()
    );
    rejects(&root, CHECK, &foreign);
}

#[test]
fn a_card_name_is_the_queue_key_and_a_number_without_leading_zero() {
    for name in NAMES {
        let (_temp, root) = queue();
        let entry = place(&root, &format!("Plan/{name}"), &card(7, ""));
        let form = format!(
            "{}: a card is a file RIG-N.md or a directory RIG-N",
            entry.display()
        );
        rejects(&root, CHECK, &form);
    }
}

#[test]
fn a_card_opens_with_its_number_and_name_and_keeps_the_card_sections() {
    for (before, after, diagnostic) in TEXTS {
        let (_temp, root) = queue();
        let path = place(&root, "Plan/RIG-7.md", &card(7, ""));
        edit(&path, before, after);
        rejects(&root, CHECK, diagnostic);
    }
}

#[test]
fn after_names_existing_cards_without_a_cycle() {
    for (cards, diagnostic) in ORDERS {
        let (_temp, root) = queue();
        for (number, after) in cards {
            let path = format!("Plan/RIG-{number}.md");
            place(&root, &path, &card(*number, after));
        }
        rejects(&root, CHECK, diagnostic);
    }
}
