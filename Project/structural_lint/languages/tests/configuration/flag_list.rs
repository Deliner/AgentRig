use super::manifest::findings;
use crate::structural_lint::languages::{analyze, tests::DETACHED};
use crate::structural_lint::rule_identities::Reference;
use std::path::Path;

// DECISION: D182
#[test]
fn yaml_flags_lint_silencing_rustflags_items_without_a_reference_above() {
    let source = "rustflags:
- -A
- warnings
- -Aunused
- --allow=dead_code
- --allow
- --cap-lints=warn
- --cap-lints
- allow
- --cfg
- -D
- warnings
other: [-A, warnings]
nested:
  rustflags: [--cfg, probe, \"--cap-lints\"]
  argv:
  - -A
";
    assert_eq!(
        findings(Reference::AllowJustification, ".agentrig/rust.yaml", source),
        [
            "2:-A",
            "4:-Aunused",
            "5:--allow=dead_code",
            "6:--allow",
            "7:--cap-lints=warn",
            "8:--cap-lints",
            "15:--cap-lints"
        ]
    );
}

// DECISION: D183
#[test]
fn yaml_flags_force_warn_rustflags_items() {
    let source = "rustflags:
- --force-warn
- clippy::panic
- --force-warn=unused_variables
- -W
- dead_code
- --warn=unused_imports
argv: [--force-warn, clippy::todo]
";
    assert_eq!(
        findings(Reference::AllowJustification, ".agentrig/rust.yaml", source),
        ["2:--force-warn", "4:--force-warn=unused_variables"]
    );
}

const LOWERING_FORMS: [&str; 12] = [
    "-A",
    "unused",
    "-Aunused",
    "--allow",
    "dead_code",
    "--allow=dead_code",
    "--cap-lints",
    "allow",
    "--cap-lints=warn",
    "--force-warn",
    "clippy::todo",
    "--force-warn=clippy::todo",
];

// DECISION: D239
#[test]
fn yaml_leaves_lowering_items_of_a_clippy_command_unread() {
    let items: String = LOWERING_FORMS
        .iter()
        .map(|item| format!("  - {item}\n"))
        .collect();
    let source = format!("clippy:\n  argv:\n  - cargo\n  - clippy\n  - --\n{items}");
    assert_eq!(
        findings(
            Reference::AllowJustification,
            ".agentrig/commands.yaml",
            &source
        ),
        Vec::<String>::new()
    );
}

#[test]
fn yaml_reports_the_line_of_a_syntax_error() {
    let source = "rustflags: []\nkey: value: other\n";
    let analysis = analyze(Path::new(".agentrig/rust.yaml"), source, DETACHED).unwrap();
    assert_eq!(analysis.parse_error, Some(2));
}
