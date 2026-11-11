use super::measured;
use crate::structural_lint::rules::reference::Reference;

#[test]
fn rust_flags_json_and_sql_templates_with_a_placeholder() {
    let source = r##"fn render(name: &str, id: u32, out: &mut String) {
    let body = format!("{{\"name\": \"{name}\"}}");
    let raw = format!(r#"[{{ "id": {} }}]"#, id);
    let lines = format!("{{\n  \"id\": {id}\n}}");
    write!(out, "SELECT * FROM users WHERE name = '{}'", name);
    writeln!(out, "  DELETE FROM jobs WHERE id = {id}");
    let set = std::format!("UPDATE t SET v = {{{}}}", name);
}
"##;
    assert_eq!(
        measured(source, Reference::InterpolatedSyntax),
        [
            (2, "format".to_string()),
            (3, "format".to_string()),
            (4, "format".to_string()),
            (5, "write".to_string()),
            (6, "writeln".to_string()),
            (7, "format".to_string())
        ]
    );
}

#[test]
fn rust_skips_fixed_templates_other_text_macros_and_test_code() {
    let source = r##"fn render(name: &str) {
    let fixed = format!("{{\"name\": \"fixed\"}}");
    let log = format!("Selected {name}");
    let lower = format!("select * from t where n = '{name}'");
    let braced = format!("{{{name}}}");
    let tail = format!("rows: SELECT {name}");
    println!("SELECT {}", name);
    // DECISION: D001
    let justified = format!("{{\"name\": \"{name}\"}}");
}
#[cfg(test)]
mod tests {
    fn fixture(name: &str) -> String {
        format!("{{\"name\": \"{name}\"}}")
    }
}
"##;
    assert!(measured(source, Reference::InterpolatedSyntax).is_empty());
}
