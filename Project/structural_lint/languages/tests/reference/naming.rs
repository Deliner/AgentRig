use crate::structural_lint::languages::analyze;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use std::path::Path;

// DECISION: D156
fn measured(source: &str, kind: Reference) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new("source.rs"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(kind))
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}

#[test]
fn rust_flags_format_strings_with_two_or_more_unnamed_placeholders() {
    let source = "fn render(out: &mut String, name: &str, size: u32) {
    let line = format!(\"{} has {}\", name, size);
    write!(out, \"{}:{:>4}\", name, size);
    std::println!(\"{0} and {0}\", name);
    let named = format!(\"{name} has {size}\");
    let single = format!(\"{}\", name);
    let escaped = format!(\"{{}} {}\", name);
    let mixed = format!(\"{name} has {}\", size);
    // DECISION: D001
    let justified = format!(\"{} {}\", name, size);
    let text = concat!(\"{}\", \"{}\");
}
#[cfg(test)]
mod tests {
    fn check() -> String {
        format!(\"{} {}\", 1, 2)
    }
}
";
    assert_eq!(
        measured(source, Reference::PositionalFormat),
        [
            (2, "format".to_string()),
            (3, "write".to_string()),
            (4, "println".to_string()),
        ]
    );
}

#[test]
fn rust_flags_a_struct_literal_field_set_from_another_field_name() {
    let source = "fn swap(x: i32, y: i32, width: u32, other: i32) {
    let crossed = Point { x: y, y: x };
    let size = Size { width, height: width };
    let own = Point { x: x, y };
    let plain = Point { x: other, y: y + 1 };
    // DECISION: D001
    let justified = Point { x: y, y: x };
}
";
    assert_eq!(
        measured(source, Reference::CrossedField),
        [(2, "x:y".to_string()), (3, "height:width".to_string())]
    );
}
