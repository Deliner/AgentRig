use super::{Consumer, INCOMPLETE, Language, messages, rust};

const UNANALYZED: [(Language, &str, &str); 6] = [
    (
        Language::Python,
        "python: [importlib]",
        "from importlib import import_module as load\nvalue = load(\"b.api\").value\n",
    ),
    (
        Language::Python,
        "python: [importlib]",
        "import importlib as loader\nvalue = loader.import_module(\"b.api\").value\n",
    ),
    (
        Language::Python,
        "python: []",
        "load = __import__\nvalue = load(\"b.api\")\n",
    ),
    (
        Language::Rust,
        "rust: [std]",
        "#[generated]\npub fn run() -> i32 { 7 }\n",
    ),
    (
        Language::Rust,
        "rust: [std]",
        "#[derive(Generated)]\npub struct Value;\npub fn run() -> i32 { 7 }\n",
    ),
    (
        Language::Rust,
        "rust: [std]",
        "#![cfg_attr(feature=\"generated\", generated)]\npub fn run() -> i32 { 7 }\n",
    ),
];

#[test]
fn unanalyzed_loaders_and_attributes_cannot_pass() {
    for (language, external, source) in UNANALYZED {
        let consumer = Consumer::new(language);
        consumer.source(source);
        consumer.external(external);
        consumer.fails(INCOMPLETE);
    }
}

#[test]
fn unresolved_python_imports_are_incomplete_at_their_line() {
    for source in ["from missing.api import value\n", "from b.api import\n"] {
        let consumer = Consumer::new(Language::Python);
        consumer.source(source);
        let found = consumer.lint();
        let located = found.iter().any(|item| {
            let incomplete = item.message.contains(INCOMPLETE);
            item.level.blocking() && incomplete && item.line == Some(1)
        });
        assert!(located, "{:?}", messages(&found));
    }
}

#[test]
fn nonexpanding_rust_metadata_remains_supported() {
    rust(
        "#![allow(dead_code)]\n#[inline]\n#[doc = \"API\"]\n\
         pub fn run() -> i32 { crate::b::value() }\n",
    )
    .passes();
}

// DECISION: D141
#[test]
fn incomplete_analysis_does_not_call_a_declared_external_unused() {
    let cases = [
        (
            Language::Python,
            "python: [unused_dep]",
            "src/a/api.py",
            "load = __import__\nvalue = load(\"b.api\")\n",
        ),
        (
            Language::Python,
            "python: [unused_dep]",
            "src/a/api.py",
            "from b.missing import value\n",
        ),
        (
            Language::Rust,
            "rust: [unused_dep]",
            "src/lib.rs",
            "pub mod a; pub mod b; pub mod missing;\n",
        ),
    ];
    for (language, external, path, source) in cases {
        let consumer = Consumer::new(language);
        consumer.write(path, source);
        consumer.external(external);
        consumer.fails(INCOMPLETE);
        let found = consumer.lint();
        let reported = messages(&found);
        let unused = reported.iter().any(|item| item.contains("unused_dep"));
        assert!(!unused, "{reported:?}");
    }
}
