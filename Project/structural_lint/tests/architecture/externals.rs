use super::{Consumer, Language};

fn unused(language: &str, files: usize) -> Vec<String> {
    vec![format!(
        "declared external {language} dependency unused_dep is not named in the {files} files \
         this rule reads under src; add the file that uses it to this rule's selection, \
         or remove the declaration"
    )]
}

fn messages(consumer: &Consumer) -> Vec<String> {
    let located = consumer.located();
    located.into_iter().map(|(_, _, message)| message).collect()
}

// DECISION: D076
// DECISION: D141
#[test]
fn declared_python_external_named_only_outside_the_selection_is_unused() {
    let consumer = Consumer::new(Language::Python);
    consumer.write("tools/build.py", "import unused_dep\n");
    consumer.source(
        "from __future__ import annotations\nimport os\nfrom b.api import value\n\n\n\
         def run():\n    return os.getpid() + value()\n",
    );
    consumer.external("python: [__future__, os, unused_dep]");
    assert_eq!(messages(&consumer), unused("Python", 2));
}

// DECISION: D076
#[test]
fn unused_rust_external_ignores_the_standard_library_seed() {
    let consumer = Consumer::new(Language::Rust);
    consumer.source(
        "use serde::Serialize;\n#[derive(Serialize)]\npub struct Row;\n\
         pub fn run() -> i32 { crate::b::value() }\n",
    );
    consumer.external("rust: [std, core, alloc, serde, unused_dep]");
    assert_eq!(messages(&consumer), unused("Rust", 3));
}
