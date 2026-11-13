use super::manifest::findings;
use crate::structural_lint::languages::analyze;
use crate::structural_lint::rule_identities::reference::Reference;
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
# DECISION: D001
- -Aclippy::pedantic
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
            "17:--cap-lints"
        ]
    );
}

// DECISION: D183
#[test]
fn yaml_flags_force_warn_rustflags_items_without_a_reference_above() {
    let source = "rustflags:
- --force-warn
- clippy::panic
- --force-warn=unused_variables
# DECISION: D001
- --force-warn
- clippy::unwrap_used
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

// DECISION: D184
#[test]
fn yaml_flags_lowering_items_before_and_after_the_deny_list_of_clippy() {
    let arguments = [
        LOWERING_FORMS.as_slice(),
        &["-D", "warnings"],
        &LOWERING_FORMS,
    ]
    .concat();
    let items: String = arguments
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
        [
            "6:-A",
            "8:-Aunused",
            "9:--allow",
            "11:--allow=dead_code",
            "12:--cap-lints",
            "14:--cap-lints=warn",
            "15:--force-warn",
            "17:--force-warn=clippy::todo",
            "20:-A",
            "22:-Aunused",
            "23:--allow",
            "25:--allow=dead_code",
            "26:--cap-lints",
            "28:--cap-lints=warn",
            "29:--force-warn",
            "31:--force-warn=clippy::todo"
        ]
    );
}

// DECISION: D184
#[test]
fn yaml_reads_only_compiler_arguments_of_a_cargo_command() {
    let source = "clippy:
  argv:
  - cargo
  - clippy
  - -A
  - --
  # DECISION: D001
  - --force-warn
  - clippy::unwrap_used
  - -W
  - clippy::print_stdout
rustc:
  argv: [cargo, +1.100.0, rustc, --lib, --, --allow, warnings]
search:
  argv: [grep, -A, '3', --, -A]
package:
  argv: [cargo, package, --allow-dirty, --, -A]
test:
  argv: [cargo, test, --, -A]
";
    assert_eq!(
        findings(
            Reference::AllowJustification,
            ".agentrig/commands.yaml",
            source
        ),
        ["13:--allow"]
    );
}

#[test]
fn yaml_reports_the_line_of_a_syntax_error() {
    let source = "rustflags: []\nkey: value: other\n";
    let analysis = analyze(Path::new(".agentrig/rust.yaml"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, Some(2));
}
