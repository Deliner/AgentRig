use super::{Consumer, Language};

const CYCLE: &str = "directory dependency cycle";
const INCLUDES: [(&str, &str, Option<&str>); 4] = [
    (
        "include!(\"api.rs\");\n",
        "pub use child::consume;\n",
        Some(CYCLE),
    ),
    (
        "include!(concat!(env!(\"OUT_DIR\"), \"/gen.rs\"));\n",
        "pub use child::Value;\n",
        None,
    ),
    ("include!(\"absent.rs\");\n", "", Some("incomplete")),
    (
        "include!(concat!(\"api\", \".rs\"));\n",
        "",
        Some("incomplete"),
    ),
];

fn parent(addition: &str) -> Consumer {
    let consumer = Consumer::empty(Language::Rust);
    let library = format!("pub mod child;\npub struct Shared;\n{addition}");
    consumer.write("src/lib.rs", &library);
    consumer.write("src/child/mod.rs", "pub fn consume(_: crate::Shared) {}\n");
    consumer.contract("src");
    consumer.contract("src/child");
    consumer
}

fn summaries(consumer: &Consumer) -> Vec<String> {
    let summary = |message: &str| {
        let head = message.split(':').next().unwrap();
        head.split(" src/").next().unwrap().to_owned()
    };
    let located = consumer.located();
    located
        .iter()
        .map(|(_, _, message)| summary(message))
        .collect()
}

#[test]
fn module_declaration_does_not_invent_use_cycles() {
    for (addition, expected) in [
        ("", vec![]),
        ("pub fn run() { child::consume(Shared); }\n", vec![CYCLE]),
        ("pub use child::consume;\n", vec![CYCLE]),
    ] {
        let consumer = parent(addition);
        consumer.compiles("src/lib.rs", &[]);
        assert_eq!(summaries(&consumer), expected);
    }
}

// DECISION: D136
#[test]
fn source_includes_compose_the_including_module() {
    for (include, usage, expected) in INCLUDES {
        let consumer = parent(usage);
        consumer.write("src/child/mod.rs", include);
        let literal = expected == Some(CYCLE);
        if literal {
            consumer.write("src/child/api.rs", "pub fn consume(_: crate::Shared) {}\n");
        }
        consumer.contract("src/child");
        match expected {
            Some(expected) => consumer.fails(expected),
            None => consumer.passes(),
        }
    }
}

#[test]
fn explicit_module_path_outside_crate_directory_is_measured() {
    let consumer = Consumer::empty(Language::Rust);
    consumer.configure("[src, 'src/**']", "[src, 'src/**', shared, 'shared/**']");
    consumer.write("shared/api.rs", "pub fn value() -> i32 { 42 }\n");
    consumer.write(
        "src/lib.rs",
        "#[path = \"../shared/api.rs\"] mod shared;\npub fn run() -> i32 { shared::value() }\n",
    );
    consumer.contract("src");
    consumer.contract("shared");
    consumer.passes();
    consumer.write(
        "shared/api.rs",
        "pub fn value() -> i32 { 42 }\npub use crate::run as back;\n",
    );
    consumer.fails(CYCLE);
}

#[test]
fn visibility_path_is_not_a_dependency() {
    for (body, expected) in [("7", vec![]), ("crate::a::seven()", vec![CYCLE])] {
        let consumer = Consumer::empty(Language::Rust);
        consumer.write("src/lib.rs", "pub mod a;\n");
        consumer.write(
            "src/a/mod.rs",
            "pub mod b;\npub fn seven() -> i32 { 7 }\npub fn run() -> i32 { b::value() }\n",
        );
        let inner = format!("pub(in crate::a) fn value() -> i32 {{ {body} }}\n");
        consumer.write("src/a/b/mod.rs", &inner);
        for directory in ["src/a/b", "src/a", "src"] {
            consumer.contract(directory);
        }
        consumer.compiles("src/lib.rs", &[]);
        assert_eq!(summaries(&consumer), expected);
    }
}
