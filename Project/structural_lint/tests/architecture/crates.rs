use super::{Consumer, rust};
use std::fs;

const ROOTS: &str = "rust_roots: [src/lib.rs, src/b/mod.rs]";
const PEER: &str = "peer: src/b/mod.rs";

const HIDDEN: [(fn(&Consumer), &str); 6] = [
    (
        |consumer| fs::remove_file(consumer.root.join("src/b/mod.rs")).unwrap(),
        "crate root is missing",
    ),
    (
        |consumer| consumer.configure(ROOTS, "rust_roots: [src/lib.rs]"),
        "must appear in rust_roots",
    ),
    (
        |consumer| consumer.configure(PEER, "peer: ../outside.rs"),
        "project-relative",
    ),
    (
        |consumer| consumer.target("pub use peer::Value;"),
        "cyclic local Rust crate reference",
    ),
    (
        |consumer| {
            consumer.configure(PEER, "peer: src/b/mod.rs\n        serde: src/b/mod.rs");
            consumer.source("#[derive(serde::Deserialize)] pub struct Value;");
        },
        "local crate macro expansion",
    ),
    (
        |consumer| consumer.source("use peer::Missing;"),
        "unresolved Rust name Missing",
    ),
];

fn peer(source: &str) -> Consumer {
    let consumer = rust(source);
    let crates = format!("{ROOTS}\n      rust_crates:\n        {PEER}");
    consumer.configure("rust_roots: [src/lib.rs]", &crates);
    consumer
}

#[test]
fn local_crate_edges_match_compiled_consumers() {
    for source in [
        "pub fn run() -> i32 { peer::value() }",
        "use peer as service; pub fn run() -> i32 { service::value() }",
        "pub fn run() -> i32 { ::peer::value() }",
    ] {
        let consumer = peer(source);
        consumer.compiles("src/b/mod.rs", &["--crate-name", "peer"]);
        consumer.compiles("src/lib.rs", &["--extern", "peer"]);
        consumer.passes();
        consumer.write("src/a/part.rs", "pub fn seven() -> i32 { 7 }\n");
        consumer.target(
            "#[path = \"../a/part.rs\"] mod part;\npub fn value() -> i32 { part::seven() }\n",
        );
        consumer.contract("src/a");
        consumer.fails("directory dependency cycle");
    }
}

#[test]
fn local_crates_cannot_hide_incomplete_analysis() {
    for (hide, expected) in HIDDEN {
        let consumer = peer("use peer::Value; pub fn run() { Value::call(); }");
        consumer.target("pub struct Value;");
        consumer.external("rust: [std, peer, serde]");
        hide(&consumer);
        consumer.rejects(expected);
    }
}
