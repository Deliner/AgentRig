use crate::structural_lint::languages::analyze;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::signature::Signature;
use std::path::Path;

// DECISION: D130
fn secrets(source: &str) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new("source.rs"), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut found: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Signature(Signature::SecretDebug))
        .map(|item| (item.line, item.symbol))
        .collect();
    found.sort();
    found
}

#[test]
fn rust_flags_secret_named_string_and_byte_fields_of_a_derived_debug_struct() {
    let source = "#[derive(Clone, Debug)]
struct Login {
    user: String,
    password: String,
    token: Option<String>,
    api_key: std::string::String,
}
/// Settings read from the environment.
#[cfg_attr(feature = \"trace\", derive(Debug))]
struct Remote<'a> {
    secret: &'a str,
    private_key: Vec<u8>,
    jwt: Option<&'static str>,
}
";
    assert_eq!(
        secrets(source),
        vec![
            (4, "Login.password".to_string()),
            (5, "Login.token".to_string()),
            (6, "Login.api_key".to_string()),
            (11, "Remote.secret".to_string()),
            (12, "Remote.private_key".to_string()),
            (13, "Remote.jwt".to_string()),
        ]
    );
}

#[test]
fn rust_skips_manual_debug_masked_types_other_names_tuple_structs_and_test_code() {
    let source = "struct Manual {
    password: String,
}
impl fmt::Debug for Manual {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(\"Manual\")
    }
}
#[derive(Debug)]
struct Masked {
    password: Redacted<String>,
    secret: Option<Redacted<String>>,
    token: Vec<u32>,
    db_password: String,
    cvv: u16,
}
#[derive(Debug)]
struct Pair(String, String);
#[cfg(test)]
mod tests {
    #[derive(Debug)]
    struct Fixture {
        password: String,
    }
}
";
    assert_eq!(secrets(source), Vec::new());
}
