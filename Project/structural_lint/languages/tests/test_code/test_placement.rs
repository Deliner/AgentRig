use crate::structural_lint::languages::analyze;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::structure::Structure;
use std::path::Path;

fn placements_in(path: &str, source: &str) -> Vec<String> {
    let analysis = analyze(Path::new(path), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Structure(Structure::TestPlacement))
        .map(|item| format!("{}:{}", item.line, item.symbol))
        .collect()
}

fn placements(source: &str) -> Vec<String> {
    placements_in("parent.rs", source)
}

// DECISION: D145
// DECISION: D223
#[test]
fn rust_flags_test_only_file_modules_outside_tests_directories() {
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
        (
            "#[cfg(test)]\n#[path = \"nested/tests.rs\"]\nmod nested;",
            vec!["3:nested/tests.rs"],
        ),
        ("#[cfg(test)]\nmod tests;", vec![]),
        (
            "#[cfg(test)]\n#[path = \"tests/world.rs\"]\nmod world_tests;",
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

// DECISION: D223
#[test]
fn test_files_outside_tests_directories_are_flagged_at_their_first_line() {
    let cases = [
        ("jobs/tests.rs", vec!["1:tests.rs"]),
        ("jobs/test_cleanup.py", vec!["1:test_cleanup.py"]),
        ("jobs/cleanup_test.py", vec!["1:cleanup_test.py"]),
        ("jobs/tests/mod.rs", vec![]),
        ("jobs/tests/tests.rs", vec![]),
        ("jobs/tests/test_cleanup.py", vec![]),
        ("tests/cleanup_test.py", vec![]),
        ("jobs/cleanup.rs", vec![]),
        ("jobs/tests.py", vec![]),
        ("jobs/conftest.py", vec![]),
        ("jobs/test_cleanup.pyi", vec![]),
        ("jobs/latest.py", vec![]),
    ];
    for (path, expected) in cases {
        assert_eq!(placements_in(path, ""), expected, "{path}");
    }
}
