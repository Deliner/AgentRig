use super::{Consumer, Language, described};
use crate::shared::testing::invoke;
use serde_json::{Value, json};
use std::fs;

const WIDE: &str = "pub fn wide() -> i32 { crate::b::value() + crate::c::value() }\n";
const HUB: &str = "src/hub/architecture.yaml";
const TEST_ONLY_USERS: [(&str, &str); 8] = [
    (
        "src/lib.rs",
        "pub mod hub; pub mod a; pub mod b; pub mod c; pub mod p;\n",
    ),
    ("src/hub/mod.rs", "pub fn value() -> i32 { 7 }\n"),
    ("src/a/mod.rs", "#[cfg(test)]\nmod tests;\n"),
    ("src/a/tests.rs", "use crate::hub::value;\n"),
    (
        "src/b/mod.rs",
        "#[cfg(test)]\nmod tests {\n    use crate::hub::value;\n}\n",
    ),
    (
        "src/c/mod.rs",
        "#[cfg(test)]\n#[path = \"tests/check.rs\"]\nmod check;\n",
    ),
    ("src/c/tests/check.rs", "use crate::hub::value;\n"),
    (
        "src/p/mod.rs",
        "pub fn run() -> i32 { crate::hub::value() }\n",
    ),
];

fn budget(consumer: &Consumer, dependents: usize) {
    let path = consumer.root.join(HUB);
    let contract = fs::read_to_string(&path).unwrap();
    fs::write(path, format!("{contract}dependents: {dependents}\n")).unwrap();
}

fn python_hub(packages: usize, filename: &str) -> Consumer {
    let consumer = Consumer::empty(Language::Python);
    let names: Vec<_> = (0..packages).map(|index| format!("p{index}")).collect();
    let mut imports = String::new();
    for name in &names {
        consumer.write(
            &format!("src/{name}/api.py"),
            "def value():\n    return 1\n",
        );
        consumer.contract(&format!("src/{name}"));
        imports += &format!("import {name}.api\n");
    }
    consumer.write(&format!("src/hub/{filename}"), &imports);
    consumer.contract("src/hub");
    consumer.contract("src");
    consumer
}

fn shared_hub() -> Consumer {
    let consumer = Consumer::empty(Language::Python);
    consumer.write("src/hub/api.py", "def value():\n    return 1\n");
    consumer.contract("src/hub");
    budget(&consumer, 1);
    for name in ["p0", "p1"] {
        consumer.write(&format!("src/{name}/use.py"), "\nimport hub.api\n");
        consumer.contract(&format!("src/{name}"));
    }
    consumer.contract("src");
    consumer
}

fn located(consumer: &Consumer, separator: &str) -> Vec<(String, Option<usize>, String)> {
    let summary = |message: String| message.split(separator).next().unwrap().to_owned();
    consumer
        .located()
        .into_iter()
        .map(|(path, line, message)| (path, line, summary(message)))
        .collect()
}

fn edges(consumer: &Consumer, args: &[&str]) -> String {
    let command = [&["lint-edges", "--config", "lint.yaml"], args].concat();
    let result = invoke(&consumer.root, &command);
    assert_eq!(result.code, Some(0), "{}", result.output());
    result.stdout
}

fn measured(consumer: &Consumer, directory: &str) -> Value {
    serde_json::from_str(&edges(consumer, &[directory, "--json"])).unwrap()
}

// DECISION: D079
// DECISION: D142
#[test]
fn dependents_budget_names_each_dependent_with_an_edge() {
    let named =
        ["p0", "p1"].map(|name| format!("src/{name} (src/{name}/use.py:2 uses src/hub/api.py)"));
    let expected = format!(
        "directory is depended on by 2 other directories (budget 1): {}",
        named.join(", ")
    );
    assert_eq!(
        located(&shared_hub(), "; "),
        [(HUB.to_owned(), None, expected)]
    );
}

// DECISION: D143
#[test]
fn edges_command_lists_the_edges_the_budget_counts() {
    let consumer = shared_hub();
    let used = |name: &str| json!([format!("src/{name}/use.py:2 uses src/hub/api.py")]);
    let expected = json!({"path": "src/hub", "rules": [{"id": "architecture",
        "incoming": {"src/p0": used("p0"), "src/p1": used("p1")}, "outgoing": {}}]});
    assert_eq!(measured(&consumer, "src/hub"), expected);
    consumer.fails("depended on by 2 other directories");
    let p0 = measured(&consumer, "src/p0");
    assert_eq!(p0["rules"][0]["outgoing"], json!({"src/hub": used("p0")}));
    let text = edges(&consumer, &["src/hub"]);
    let listed =
        "  incoming: 2 directories\n    src/p0\n      src/p0/use.py:2 uses src/hub/api.py\n";
    assert!(text.contains(listed), "{text}");
    let unselected = invoke(
        &consumer.root,
        &["lint-edges", "--config", "lint.yaml", "docs"],
    );
    assert_ne!(unselected.code, Some(0));
    assert!(
        unselected.stderr.contains("selects docs as a directory"),
        "{}",
        unselected.stderr
    );
}

// DECISION: D146
#[test]
fn dependents_budget_counts_only_production_uses() {
    let consumer = Consumer::empty(Language::Rust);
    for (path, text) in TEST_ONLY_USERS {
        consumer.write(path, text);
    }
    for directory in [
        "src/hub",
        "src/a",
        "src/b",
        "src/c/tests",
        "src/c",
        "src/p",
        "src",
    ] {
        consumer.contract(directory);
    }
    budget(&consumer, 0);
    let production = "src/p/mod.rs:1 uses src/hub/mod.rs";
    let expected =
        format!("directory is depended on by 1 other directories (budget 0): src/p ({production})");
    assert_eq!(located(&consumer, "; "), [(HUB.to_owned(), None, expected)]);
    let hub = measured(&consumer, "src/hub");
    assert_eq!(hub["rules"][0]["incoming"], json!({"src/p": [production]}));
}

// DECISION: D121
#[test]
fn fan_out_default_budget_flags_files_above_ten_directories() {
    python_hub(10, "api.py").passes();
    let over = "file uses 11 other project directories (budget 10)";
    assert_eq!(
        located(&python_hub(11, "api.py"), ":"),
        [("src/hub/api.py".to_owned(), None, over.to_owned())]
    );
}

// DECISION: D121
#[test]
fn fan_out_exempts_python_main_entry_point() {
    python_hub(11, "__main__.py").passes();
}

// DECISION: D121
#[test]
fn fan_out_setting_flags_wide_files_but_not_crate_roots() {
    let over = "file uses 2 other project directories (budget 1): src/b, src/c";
    for (hub, expected) in [("src/a/mod.rs", vec![over]), ("src/lib.rs", vec![])] {
        let consumer = Consumer::new(Language::Rust);
        consumer.write("src/c/mod.rs", "pub fn value() -> i32 { 3 }\n");
        consumer.write("src/lib.rs", "pub mod a; pub mod b; pub mod c;\n");
        let source = fs::read_to_string(consumer.root.join(hub)).unwrap();
        consumer.write(hub, &(source + WIDE));
        consumer.contract("src/c");
        consumer.contract("src");
        let setting = "rust_roots: [src/lib.rs]\n      fan_out: 1\n";
        consumer.configure("rust_roots: [src/lib.rs]\n", setting);
        let expected: Vec<_> = expected
            .into_iter()
            .map(|message| (hub.to_owned(), None, message.to_owned()))
            .collect();
        assert_eq!(located(&consumer, ";"), expected);
    }
    assert!(described().get("fan_out").is_some());
}
