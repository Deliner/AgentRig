use crate::structural_lint::languages::analyze;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::structure::Structure;
use std::path::Path;

fn test_code_in(path: &str, source: &str) -> Vec<String> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Structure(Structure::TestInCodeFile))
        .map(|item| format!("{}:{}", item.line, item.symbol))
        .collect()
}

// DECISION: D224
#[test]
fn only_a_bodiless_test_module_in_a_directory_entry_is_accepted() {
    let cases = [
        (
            "foo.rs",
            "#[cfg(test)]\nmod tests {\n    #[test]\n    fn runs() {}\n}",
            vec!["2:tests"],
        ),
        (
            "foo.rs",
            "#[cfg(test)]\n#[path = \"tests/foo.rs\"]\nmod tests;",
            vec!["3:tests"],
        ),
        ("foo.rs", "#[cfg(test)]\nfn helper() {}", vec!["2:helper"]),
        ("mod.rs", "#[cfg(test)]\nmod tests;", vec![]),
        ("src/lib.rs", "#[cfg(test)]\nmod tests;", vec![]),
        ("src/main.rs", "#[cfg(test)]\nmod tests;", vec![]),
        ("mod.rs", "#[cfg(test)]\nmod tests {}", vec!["2:tests"]),
        ("mod.rs", "#[cfg(test)]\nfn helper() {}", vec!["2:helper"]),
    ];
    for (path, source, expected) in cases {
        assert_eq!(test_code_in(path, source), expected, "{path}: {source}");
    }
}

// DECISION: D224
#[test]
fn each_outermost_test_item_is_reported_once() {
    let cases = [
        ("#[test]\nfn lone() {}", vec!["2:lone"]),
        ("#[tokio::test]\nasync fn io() {}", vec!["2:io"]),
        (
            "#[cfg(all(test, feature = \"slow\"))]\nuse super::shared;",
            vec!["2:use_declaration"],
        ),
        ("#[cfg(test)]\n#[test]\nfn twice() {}", vec!["3:twice"]),
        (
            "#[cfg(test)]\n// fixture\nfn fixture() {}",
            vec!["3:fixture"],
        ),
        (
            "mod inner {\n    #[cfg(test)]\n    fn helper() {}\n}",
            vec!["3:helper"],
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(test_code_in("foo.rs", source), expected, "{source}");
    }
}

// DECISION: D224
#[test]
fn code_compiled_outside_tests_or_steered_at_run_time_is_not_test_code() {
    let sources = [
        "#[cfg(not(test))]\nfn real() {}",
        "#[cfg(feature = \"test-fixtures\")]\nmod fixtures;",
        "fn real() -> bool {\n    cfg!(test)\n}",
        "#![cfg(test)]\nfn inner() {}",
        "#[cfg(test)]",
    ];
    for source in sources {
        assert_eq!(
            test_code_in("foo.rs", source),
            Vec::<String>::new(),
            "{source}"
        );
    }
}
