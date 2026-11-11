use super::{Consumer, Language, messages};
use serde_yaml_ng::Value;
use std::fs;

const DOCS: &str = "src/docs/architecture.yaml";

const VIOLATIONS: [(&str, fn(&mut Value)); 6] = [
    (DOCS, |data| {
        drop(data["files"].as_mapping_mut().unwrap().remove("guide.md"))
    }),
    ("src/architecture.yaml", |data| {
        drop(data["directories"].as_mapping_mut().unwrap().remove("docs"))
    }),
    (DOCS, |data| {
        data["files"]["deleted.md"] = "Removed resource".into()
    }),
    (DOCS, |data| {
        data["directories"]["guide.md"] = "Wrong kind".into()
    }),
    (DOCS, |data| {
        data["purpose"] = "First line\nSecond line".into()
    }),
    (DOCS, |data| data["files"]["guide.md"] = "  ".into()),
];

pub fn documents() -> Consumer {
    let consumer = Consumer::empty(Language::Python);
    consumer.write("src/docs/guide.md", "Feature ownership guide\n");
    consumer.contract("src/docs");
    consumer.contract("src");
    consumer
}

fn edit(consumer: &Consumer, contract: &str, change: fn(&mut Value)) {
    let path = consumer.root.join(contract);
    let mut data: Value = serde_yaml_ng::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    change(&mut data);
    fs::write(path, serde_yaml_ng::to_string(&data).unwrap()).unwrap();
}

fn reported_only_at(consumer: &Consumer, contract: &str) {
    let found = consumer.lint();
    let blocking = found.iter().any(|item| item.level.blocking());
    let located = found.iter().all(|item| item.path == contract);
    assert!(blocking && located, "{contract}: {:?}", messages(&found));
}

#[test]
fn source_free_inventory_contracts() {
    documents().passes();
    let missing = documents();
    fs::remove_file(missing.root.join(DOCS)).unwrap();
    reported_only_at(&missing, DOCS);
    for (contract, change) in VIOLATIONS {
        let consumer = documents();
        edit(&consumer, contract, change);
        reported_only_at(&consumer, contract);
    }
}

#[test]
fn inventory_excludes_generated_subtree() {
    let consumer = documents();
    consumer.write("src/docs/generated/output.txt", "Generated output\n");
    let exclude = "exclude: ['src/docs/generated', 'src/docs/generated/**']\nversion: 1\n";
    consumer.configure("version: 1\n", exclude);
    consumer.passes();
}

#[test]
fn empty_directory_requires_its_own_contract() {
    let consumer = documents();
    fs::create_dir(consumer.root.join("src/docs/drafts")).unwrap();
    consumer.contract("src/docs");
    let found = consumer.lint();
    let blocking = found.iter().any(|item| item.level.blocking());
    let drafts = "src/docs/drafts/architecture.yaml";
    let named = found.iter().any(|item| item.path == drafts);
    assert!(blocking && named, "{:?}", messages(&found));
    consumer.contract("src/docs/drafts");
    consumer.passes();
}

#[test]
fn rule_exclusion_keeps_inventory_and_analysis_in_scope() {
    let consumer = documents();
    consumer.write("src/docs/generated.py", "from unknown import missing\n");
    consumer.write("src/docs/external/guide.md", "External documentation\n");
    let exclude =
        "    exclude: ['src/docs/generated.py', 'src/docs/external', 'src/docs/external/**']\n";
    consumer.configure("    extensions:", &format!("{exclude}    extensions:"));
    consumer.passes();
}
