use super::{INCOMPLETE, rust};
use crate::shared::testing::{git, invoke};

const EMBEDDED: [&str; 4] = [
    "include_str!(\"../b/data.txt\")",
    "include_bytes!(r#\"../b/data.txt\"#)",
    "text!(\"../b/data.txt\")",
    "format!(\"{}\", include_str!(\"../b/data.txt\"))",
];

#[test]
fn embedded_resources_are_edges_to_their_directory() {
    for expression in EMBEDDED {
        let consumer = rust(&format!(
            "use std::include_str as text; pub fn run() -> i32 {{ {expression}.len() as i32 }}"
        ));
        consumer.write("src/b/data.txt", "content");
        consumer.contract("src/b");
        consumer.compiles("src/lib.rs", &[]);
        consumer.passes();
        consumer.cycles();
    }
}

#[test]
fn unverified_resource_targets_are_incomplete() {
    for argument in [
        "\"../b/missing.txt\"",
        "concat!(\"../b/\", \"data.txt\")",
        "\"../../../outside.txt\"",
    ] {
        rust(&format!(
            "pub fn run() {{ let _ = include_str!({argument}); }}"
        ))
        .fails(INCOMPLETE);
    }
}

#[test]
fn ignored_resource_targets_are_incomplete() {
    let consumer = rust("pub fn run() { let _ = include_str!(\"../b/data.txt\"); }");
    consumer.write("src/b/data.txt", "content");
    consumer.write(".gitignore", "src/b/data.txt\n");
    git(&consumer.root, &["init", "-q"]);
    let result = invoke(&consumer.root, &["lint", "--config", "lint.yaml", "--json"]);
    assert_eq!(result.code, Some(1), "{}", result.output());
    assert!(result.stdout.contains(INCOMPLETE), "{}", result.stdout);
}
