use super::{INCOMPLETE, declared};

const REACHING: [(&str, &str); 7] = [
    (
        "[std]",
        "pub fn run() -> i32 { assert!(crate::b::value() == 7); 7 }",
    ),
    (
        "[std]",
        "pub fn run() -> i32 { assert_eq!(format!(\"{}\", crate::b::value()), \"7\"); 7 }",
    ),
    (
        "[std]",
        "use crate::b as format; \
             pub fn run() -> i32 { assert_eq!(format!(\"{}\", format::value()), \"7\"); 7 }",
    ),
    (
        "[std, anyhow]",
        "use anyhow::ensure; pub fn run() { ensure!(crate::b::value() == 7); }",
    ),
    (
        "[std, serde_json]",
        "use serde_json::json; \
             pub fn run() { let _ = json!({\"key\": [crate::b::value()]}); }",
    ),
    (
        "[std]",
        "macro_rules! value { ($x:expr) => { $crate::b::value() + $x }; } \
             pub fn run() -> i32 { value!(0) }",
    ),
    (
        "[std]",
        "macro_rules! make { ($name:ident) => { pub struct $name; \
             impl $name { pub fn get() -> i32 { crate::b::value() } } }; } \
             make!(Made); pub fn run() -> i32 { self::Made::get() }",
    ),
];

#[test]
fn macro_arguments_and_definitions_reach_local_items() {
    for (external, source) in REACHING {
        let consumer = declared(external, source);
        consumer.passes();
        consumer.cycles();
    }
}

// DECISION: D136
#[test]
fn macros_imported_from_external_crates_stay_external() {
    for source in [
        "use custom::assert; pub fn run() { assert!(true); }",
        "use custom::*; pub fn run() { assert!(true); }",
    ] {
        declared("[std, custom]", source).passes();
    }
}

#[test]
fn macros_of_unknown_provenance_are_incomplete() {
    for source in [
        "pub fn run() { generate!(crate::b::value()); }",
        "pub fn run() { use custom::assert; assert!(true); }",
        "pub fn run() { assert!({ use custom::assert; assert!(true); true }); }",
    ] {
        declared("[std, anyhow, serde_json, custom]", source).fails(INCOMPLETE);
    }
}
