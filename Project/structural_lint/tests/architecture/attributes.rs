use super::{INCOMPLETE, declared, rust};

// DECISION: D136
#[test]
fn attributes_keep_the_dependencies_they_name_or_guard() {
    let checks = "mod checks { pub fn run() -> i32 { crate::b::value() } }";
    let cases = [
        (
            "[std, serde]",
            "#[derive(serde::Deserialize)] pub struct Value { \
             #[serde(default = \"crate::b::value\")] x: i32 }"
                .to_owned(),
        ),
        ("[std]", format!("#[cfg(test)] {checks}")),
        (
            "[std]",
            format!("#[cfg(not(target_os = \"none\"))] {checks}"),
        ),
        ("[std]", format!("#[cfg(feature = \"extra\")] {checks}")),
        (
            "[std]",
            "#[unsafe(no_mangle)] pub extern \"C\" fn run() -> i32 { crate::b::value() } \
             #[export_name = \"other\"] pub fn other() {}"
                .to_owned(),
        ),
        (
            "[std, wasm_bindgen]",
            "use wasm_bindgen::prelude::*; \
             #[wasm_bindgen] pub fn run() -> i32 { crate::b::value() }"
                .to_owned(),
        ),
        (
            "[std]",
            "#![expect(unused)] #[expect(dead_code)] pub fn run() -> i32 { crate::b::value() }"
                .to_owned(),
        ),
    ];
    for (external, source) in cases {
        let consumer = declared(external, &source);
        consumer.passes();
        consumer.cycles();
    }
}

#[test]
fn attribute_string_labels_are_not_dependencies() {
    let consumer = declared(
        "[std, serde]",
        "#[derive(serde::Deserialize)] pub struct Value { \
         #[serde(rename = \"crate::missing::value\")] x: i32 }",
    );
    consumer.passes();
    consumer.back_edge();
    consumer.passes();
}

#[test]
fn unanalyzed_attributes_are_incomplete() {
    for source in [
        "#[derive(serde::Deserialize)] #[serde(unknown = \"value\")] pub struct Value;",
        "#[derive(custom::Debug)] pub struct Value;",
        "use custom::Debug; #[derive(Debug)] pub struct Value;",
        "#[crate::b::attribute] pub fn run() {}",
    ] {
        declared("[std, serde, custom]", source).fails(INCOMPLETE);
    }
}

#[test]
fn standard_derives_compile_without_dependencies() {
    let consumer = rust(
        "#[derive(Debug, Clone, Default)] pub struct Value; \
         pub fn run() -> i32 { let _ = format!(\"{:?}\", Value); 7 }",
    );
    consumer.compiles("src/lib.rs", &[]);
    consumer.passes();
}
