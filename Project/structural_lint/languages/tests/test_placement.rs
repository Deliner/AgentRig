use super::super::analyze;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::structure::Structure;
use std::path::Path;

fn placements(source: &str) -> Vec<String> {
    let analysis = analyze(Path::new("parent.rs"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Structure(Structure::TestPlacement))
        .map(|item| format!("{}:{}", item.line, item.symbol))
        .collect()
}

// DECISION: D145
#[test]
fn rust_flags_test_only_file_modules_outside_tests_files_and_directories() {
    let cases = [
        ("#[cfg(test)]\nmod mesh_tests;", vec!["2:mesh_tests"]),
        (
            "#[cfg(test)]\n#[path = \"local_tests.rs\"]\nmod tests;",
            vec!["3:local_tests.rs"],
        ),
        (
            "#[cfg(all(test, feature = \"slow\"))]\n// slow\npub(crate) mod slow;",
            vec!["3:slow"],
        ),
        (
            "#[cfg(test)]\nmod checks {\n    mod helpers;\n}",
            vec!["3:helpers"],
        ),
        ("#[cfg(test)]\nmod tests;", vec![]),
        (
            "#[cfg(test)]\n#[path = \"tests/world.rs\"]\nmod world_tests;",
            vec![],
        ),
        (
            "#[cfg(test)]\n#[path = \"nested/tests.rs\"]\nmod nested;",
            vec![],
        ),
        ("#[cfg(test)]\nmod tests {\n    mod fixtures;\n}", vec![]),
        (
            "#[cfg(test)]\nmod checks {\n    #[test]\n    fn runs() {}\n}",
            vec![],
        ),
        (
            "mod real;\n#[cfg(feature = \"test-fixtures\")]\nmod fixtures;\n#[cfg(not(test))]\nmod other;",
            vec![],
        ),
    ];
    for (source, expected) in cases {
        assert_eq!(placements(source), expected, "{source}");
    }
}
