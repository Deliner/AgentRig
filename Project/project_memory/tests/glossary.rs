use super::consumer::{accepts, consumer, rejects};
use std::{fs, path::PathBuf};
use tempfile::TempDir;

const CHECK: [&str; 1] = ["memory-check"];

fn glossary(text: &str) -> (TempDir, PathBuf, String) {
    let (temp, root) = consumer();
    let path = root.join("notes/Glossary.md");
    fs::write(&path, text).unwrap();
    let shown = path.display().to_string();
    (temp, root, shown)
}

#[test]
fn a_header_with_entries_of_distinct_words_passes() {
    let text =
        "# Glossary\n\n- **очередь** — очередь трекера\n\n- **VAC** — проверенное изменение\n";
    let (_temp, root, _) = glossary(text);
    accepts(&root);
}

#[test]
fn a_line_that_is_not_an_entry_names_the_file_the_line_and_the_entry_form() {
    let malformed = [
        "очередь: трекер",
        "- **очередь** —",
        "- **очередь** — ",
        "- **** — трекер",
    ];
    for line in malformed {
        let (_temp, root, shown) = glossary(&format!("# Glossary\n\n{line}\n"));
        let named = format!("{shown}:3: a glossary entry is `- **word** — meaning`");
        rejects(&root, &CHECK, &named);
    }
}

#[test]
fn a_glossary_without_a_header_names_its_first_line() {
    let (_temp, root, shown) = glossary("- **очередь** — очередь трекера\n");
    let named = format!("{shown}:1: a glossary starts with a `# ` header line");
    rejects(&root, &CHECK, &named);
}

#[test]
fn a_word_repeated_in_another_letter_case_names_both_lines_and_the_word() {
    let text = "# Glossary\n- **очередь** — очередь трекера\n\n- **Очередь** — порядок работ\n";
    let (_temp, root, shown) = glossary(text);
    let named = format!("{shown}:2 and {shown}:4 define the word Очередь twice");
    rejects(&root, &CHECK, &named);
}
