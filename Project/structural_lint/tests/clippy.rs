// DECISION: D239
use super::consumer::parsed;
use crate::shared::testing::{FAIL, GATE, REPAIR, invoke, project, write};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};
use tempfile::TempDir;

const MANIFEST: &str = "[package]\nname = \"probe\"\nversion = \"0.1.0\"\nedition = \"2024\"\n";
const RUST: &str = "rustflags: []\nmanifest: Cargo.toml\n";
const JUSTIFIED: &str = "- id: allow-justification
  kind: allow-justification
  target: file
  include: [src/**]
  level: error
  extensions: [.rs]
  warning_skill: guides/repair/SKILL.md
  error_skill: guides/repair/SKILL.md
";
const FLAWED: &str = "pub fn narrow(value: u64) -> u32 {
    let unused = 1;
    value as u32
}
pub fn parse(text: &str) -> u32 {
    text.parse().unwrap()
}
";
const GUIDES: [(&str, &str); 3] = [
    (
        "clippy::as_conversions",
        "guides/convert-without-as/SKILL.md",
    ),
    ("clippy::unwrap_used", "guides/handle-panic/SKILL.md"),
    ("unused_variables", REPAIR),
];
const FIXED: &str = "pub fn narrow(value: u64) -> Option<u32> {
    u32::try_from(value).ok()
}
";
const NARROWING: &str = "pub fn narrow(value: u64) -> u32 {
    value as u32
}
";
const UNUSED: &str = "fn main() {
    let unused = 1;
}
";
const EXPECTATION: &str = "#![expect(clippy::as_conversions)]\n";
const EXCEPTION: &str = "// DECISION: D001 allow-justification\n";
const DIVISION: &str = "pub fn half(value: u64) -> u64 {
    value / 2
}
";
const DENIED: &str = "\n[lints.clippy]\ninteger_division = \"deny\"\n";

fn rust_project(sources: &[(&str, &str)]) -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    project(&root, FAIL, GATE);
    let lint = root.join("lint.yaml");
    let rules = fs::read_to_string(&lint).unwrap() + JUSTIFIED;
    fs::write(lint, rules).unwrap();
    write(&root, ".agentrig/rust.yaml", RUST);
    write(&root, "Cargo.toml", MANIFEST);
    for (path, text) in sources {
        write(&root, path, text);
    }
    (temp, root)
}

fn lint(root: &Path) -> (Option<i32>, Vec<Value>) {
    let result = invoke(root, &["lint", "--json"]);
    (result.code, parsed(&result))
}

fn located(found: &[Value]) -> Vec<String> {
    let place = |item: &Value| {
        let rule = item["rule"].as_str().unwrap();
        let path = item["path"].as_str().unwrap();
        let line = item["line"].as_u64();
        let line = line.map_or_else(String::new, |line| format!(":{line}"));
        format!("{rule} {path}{line}")
    };
    let mut places: Vec<String> = found.iter().map(place).collect();
    places.sort();
    places
}

fn configured(found: &[Value]) -> bool {
    let skill = |item: &Value| item["skill"].as_str().unwrap().ends_with(REPAIR);
    found.iter().all(skill)
}

// DECISION: D240
#[test]
fn lint_reports_each_clippy_message_as_an_error_with_the_skill_that_repairs_it() {
    let (_temp, root) = rust_project(&[("src/lib.rs", FLAWED)]);
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{found:?}");
    let expected = [
        "clippy::as_conversions src/lib.rs:3",
        "clippy::unwrap_used src/lib.rs:6",
        "unused_variables src/lib.rs:2",
    ];
    assert_eq!(located(&found), expected);
    let guided = found.iter().all(|item| {
        let rule = item["rule"].as_str().unwrap();
        let skill = item["skill"].as_str().unwrap();
        let guide = GUIDES.iter().find(|(listed, _)| *listed == rule);
        let named = guide.is_some_and(|(_, guide)| skill.ends_with(guide));
        item["level"] == "error" && named
    });
    assert!(guided, "{found:?}");
    write(&root, "src/lib.rs", FIXED);
    assert_eq!(lint(&root), (Some(0), Vec::new()));
}

#[test]
fn clippy_checks_the_binaries_and_examples_but_not_the_tests() {
    let sources = [
        ("src/lib.rs", FIXED),
        ("src/main.rs", UNUSED),
        ("examples/demo.rs", UNUSED),
        ("tests/outside.rs", NARROWING),
    ];
    let (_temp, root) = rust_project(&sources);
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{found:?}");
    let expected = [
        "unused_variables examples/demo.rs:2",
        "unused_variables src/main.rs:2",
    ];
    assert_eq!(located(&found), expected);
}

#[test]
fn a_package_without_a_library_is_checked_through_its_binaries() {
    let (_temp, root) = rust_project(&[("src/main.rs", UNUSED)]);
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{found:?}");
    assert_eq!(located(&found), ["unused_variables src/main.rs:2"]);
}

#[test]
fn lint_names_why_clippy_could_not_check_the_project() {
    let (_temp, root) = rust_project(&[("src/lib.rs", FIXED)]);
    let missing = RUST.replace("Cargo.toml", "missing/Cargo.toml");
    write(&root, ".agentrig/rust.yaml", &missing);
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{found:?}");
    assert_eq!(located(&found), ["clippy missing/Cargo.toml"]);
    let cause = found[0]["message"].as_str().unwrap();
    assert!(cause.contains("does not exist"), "{cause}");
    assert!(configured(&found), "{found:?}");
    write(&root, ".agentrig/rust.yaml", RUST);
    write(&root, "src/lib.rs", "pub fn broken( {\n");
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{found:?}");
    let places = located(&found);
    let unparsed = places
        .iter()
        .any(|place| place.starts_with("clippy src/lib.rs:"));
    assert!(unparsed && configured(&found), "{found:?}");
}

#[test]
fn a_clippy_suppression_passes_only_with_its_decision_reference() {
    let suppressed = format!("{EXPECTATION}{NARROWING}");
    let (_temp, root) = rust_project(&[("src/lib.rs", &suppressed)]);
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{found:?}");
    assert_eq!(located(&found), ["allow-justification src/lib.rs:1"]);
    write(&root, "src/lib.rs", &format!("{EXCEPTION}{suppressed}"));
    assert_eq!(lint(&root), (Some(0), Vec::new()));
}

#[test]
fn a_lint_the_manifest_denies_is_reported_beyond_the_profile() {
    let (_temp, root) = rust_project(&[("src/lib.rs", DIVISION)]);
    assert_eq!(lint(&root), (Some(0), Vec::new()));
    write(&root, "Cargo.toml", &format!("{MANIFEST}{DENIED}"));
    let (code, found) = lint(&root);
    assert_eq!(code, Some(1), "{found:?}");
    assert_eq!(located(&found), ["clippy::integer_division src/lib.rs:2"]);
    assert!(configured(&found), "{found:?}");
}
