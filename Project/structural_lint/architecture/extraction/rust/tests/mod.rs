use crate::structural_lint::architecture::extraction::rust::conditions::supported;

#[test]
fn platform_feature_and_test_predicates_are_supported() {
    for condition in [
        "(test)",
        "(unix)",
        "(target_os=\"macos\")",
        "(target_family=\"unix\")",
        "(not(target_os=\"linux\"))",
        "(all(unix,not(any(target_os=\"macos\",target_os=\"ios\"))))",
        "(feature=\"extra\")",
        "(feature=\"serde-json\")",
        "(any(unix,feature=\"extra\"))",
        "(not(feature=\"Std_2.x+alloc\"))",
        "(all(feature=\"native\",not(target_family=\"wasm\")))",
    ] {
        assert!(supported(condition), "{condition}");
    }
}

#[test]
fn unknown_and_malformed_predicates_are_unsupported() {
    for condition in [
        "(debug_assertions)",
        "(target_arch=\"x86_64\")",
        "(feature=extra)",
        "(feature=\"\")",
        "(feature=\"a b\")",
        "(not(unix,windows))",
        "(all(unix,))",
        "(target_os=linux)",
        "(target_os=\"Linux\")",
        "(all(unix)",
        "test",
    ] {
        assert!(!supported(condition), "{condition}");
    }
}
