use crate::structural_lint::languages::analyze;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::reference::Reference;
use std::path::Path;

// DECISION: D098
pub(super) fn findings(kind: Reference, path: &str, source: &str) -> Vec<String> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(kind))
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
        findings(Reference::AllowJustification, "Cargo.toml", source),
        [
            "1:private_intra_doc_links",
            "5:unused",
            "8:missing_docs",
            "12:pedantic",
            "15:unused_imports"
        ]
    );
}

// DECISION: D182
#[test]
fn toml_flags_every_rustflags_key_in_any_table() {
    let source = "build.rustflags = \"--cap-lints allow\"
[target.x86_64-unknown-linux-gnu]
# DECISION: D001
rustflags = [\"-C\", \"target-cpu=native\"]
[target.'cfg(unix)']
linker = \"cc\"
rustflags = []
[env]
RUSTFLAGS = \"-Awarnings\"
[profile.dev]
package.probe = { rustflags = [\"-Zshare-generics\"] }
";
    assert_eq!(
        findings(Reference::CargoRustflags, ".cargo/config.toml", source),
        ["1:rustflags", "4:rustflags", "7:rustflags", "11:rustflags"]
    );
}

#[test]
fn toml_reports_the_line_of_a_syntax_error() {
    let source = "[package]\nname = \"probe\"\nname = \"twice\"\n";
    let analysis = analyze(Path::new("Cargo.toml"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, Some(3));
}
