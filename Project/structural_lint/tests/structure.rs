use super::consumer::write;
use crate::shared::testing::{Run, launch, run, worker};
use std::path::Path;

const CROWD: [&str; 12] = [
    "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india", "juliet",
    "kilo", "lima",
];

fn repository(root: &Path) {
    write(
        root,
        "lib.rs",
        "pub mod engine;\npub mod gadget;\npub mod reader;\npub mod spooler;\n",
    );
    described_directories(root);
    for word in CROWD {
        write(
            root,
            &format!("crowd/{word}.rs"),
            &format!("pub fn {word}() {{}}\n"),
        );
    }
    for twin in ["twin_one", "twin_two"] {
        write(
            root,
            &format!("{twin}.rs"),
            "pub fn mirror_value() -> u8 { 7 }\n",
        );
    }
    let mixed = "pub struct Apple;\nimpl Apple { pub fn grow() -> Apple { Apple } }\n\
        pub struct Berry;\nimpl Berry { pub fn ripen() -> Berry { Berry } }\n";
    write(root, "mixed.rs", mixed);
    write(root, "plain.rs", "pub fn quiet_kettle() {}\n");
}

fn described_directories(root: &Path) {
    write(
        root,
        "reader/architecture.yaml",
        "purpose: Reader of bytes, frames, pages and lines\n",
    );
    write(root, "reader/mod.rs", "pub fn fetch() {}\n");
    write(
        root,
        "gadget/architecture.yaml",
        "purpose: Parse settings tokens\n",
    );
    write(root, "gadget/mod.rs", "pub fn widget() {}\n");
    write(root, "spooler/architecture.yaml", "purpose: Spool bytes\n");
    write(root, "spooler/mod.rs", "pub mod bytes;\npub mod spool;\n");
    write(root, "spooler/spool.rs", "pub fn wind_reel() {}\n");
    write(root, "spooler/bytes.rs", "pub fn count_octets() {}\n");
    write(
        root,
        "engine/architecture.yaml",
        "purpose: Schedule ticks and render frames\n",
    );
    write(root, "engine/mod.rs", "pub mod one;\npub mod two;\n");
    for (part, word) in [("one", "north"), ("two", "south")] {
        write(
            root,
            &format!("engine/{part}.rs"),
            &format!("pub fn {word}() {{}}\n"),
        );
    }
}

fn structure(root: &Path, path: &str) -> Run {
    run(&mut launch(worker(), root, &["lint-structure", path]), "")
}

fn rows(root: &Path) -> Vec<(String, String, String)> {
    let result = structure(root, ".");
    assert_eq!(result.code, Some(0), "{}", result.output());
    let lines = result.stdout.lines().skip(1);
    lines
        .map(|line| {
            let mut parts = line.split_whitespace();
            let mut next = || parts.next().unwrap_or_default().to_owned();
            (next(), next(), next())
        })
        .collect()
}

fn row(score: &str, kind: &str, path: &str) -> (String, String, String) {
    (score.into(), kind.into(), path.into())
}

#[test]
fn structure_reports_the_edit_kind_of_each_problem() {
    let temp = tempfile::tempdir().unwrap();
    repository(temp.path());
    let found = rows(temp.path());
    for expected in [
        row("100", "split", "gadget/"),
        row("20", "split", "reader/"),
        row("100", "rename", "engine/"),
        row("100", "merge", "twin_one.rs"),
        row("100", "merge", "twin_two.rs"),
        row("33", "split", "mixed.rs"),
        row("20", "split", "crowd/"),
        row("0", "ok", "spooler/"),
        row("0", "undescribed", "plain.rs"),
        row("0", "undescribed", "."),
    ] {
        assert!(found.contains(&expected), "{expected:?} in {found:?}");
    }
}

#[test]
fn structure_output_is_identical_across_runs_and_locations() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let elsewhere = second.path().join("deeper/other");
    repository(first.path());
    repository(&elsewhere);
    let baseline = structure(first.path(), ".").stdout;
    assert!(baseline.starts_with("total "), "{baseline}");
    assert_eq!(structure(first.path(), ".").stdout, baseline);
    assert_eq!(structure(&elsewhere, ".").stdout, baseline);
}

#[test]
fn structure_reads_rust_documentation_when_no_contract_describes_a_unit() {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), "lib.rs", "pub mod ast;\n");
    let documented = "//! Abstract syntax trees.\n//!\n//! Later paragraph.\npub fn node() {}\n";
    write(temp.path(), "ast/mod.rs", documented);
    assert!(rows(temp.path()).contains(&row("100", "split", "ast/")));
}

#[test]
fn structure_rejects_a_missing_or_file_path() {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), "lib.rs", "pub fn plain() {}\n");
    for path in ["missing", "lib.rs"] {
        let result = structure(temp.path(), path);
        assert_eq!(result.code, Some(2), "{path}: {}", result.output());
    }
}
