use crate::structural_lint::languages::analyze;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use std::path::Path;

// DECISION: D098
fn allow_findings(path: &str, source: &str) -> Vec<String> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::AllowJustification))
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
        .into_iter()
        .map(|(line, symbol)| format!("{line}:{symbol}"))
        .collect()
}

#[test]
fn toml_flags_allow_levels_under_lint_tables_without_a_reference_above() {
    let source = "lints.rustdoc.private_intra_doc_links = \"allow\"
[package]
name = \"probe\"
[lints.rust]
unused = \"allow\"
# DECISION: D001
dead_code = \"allow\"
missing_docs = { level = \"allow\", priority = 1 }
unsafe_code = \"deny\"
[lints.clippy]
# DECISION: D001 because
pedantic = \"allow\"
all = { level = \"warn\", priority = -1 }
[workspace.lints.rust]
unused_imports = \"allow\"
[dependencies]
allow = \"1\"
";
    assert_eq!(
        allow_findings("Cargo.toml", source),
        [
            "1:private_intra_doc_links",
            "5:unused",
            "8:missing_docs",
            "12:pedantic",
            "15:unused_imports"
        ]
    );
}

#[test]
fn toml_reports_the_line_of_a_syntax_error() {
    let source = "[package]\nname = \"probe\"\nname = \"twice\"\n";
    let analysis = analyze(Path::new("Cargo.toml"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, Some(3));
}
