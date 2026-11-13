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

#[test]
fn yaml_reports_the_line_of_a_syntax_error() {
    let source = "rustflags: []\nkey: value: other\n";
    let analysis = analyze(Path::new(".agentrig/rust.yaml"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, Some(2));
}
