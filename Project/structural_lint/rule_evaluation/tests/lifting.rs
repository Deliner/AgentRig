// DECISION: D222
use crate::structural_lint::rule_evaluation::evaluation::evaluate;
use crate::structural_lint::rule_set::lint_config::Config;
use std::fs;

const KINDS: [&str; 5] = [
    "comment-reference",
    "panic-path",
    "positional-format",
    "same-type-parameters",
    "debug-artifact",
];

fn found(source: &str) -> Vec<(String, Option<usize>)> {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("lib.rs"), source).unwrap();
    let rules = KINDS.map(|kind| {
        format!(
            "- {{id: {kind}, kind: {kind}, target: file, include: ['**'], extensions: [.rs], \
             level: error, warning_skill: s, error_skill: s}}\n"
        )
    });
    let text = format!("version: 1\nconfig_skill: s\nrules:\n{}", rules.concat());
    let config: Config = crate::shared::strict_yaml::decode(&text).unwrap();
    let mut found: Vec<_> = evaluate(root.path(), &config)
        .unwrap()
        .into_iter()
        .map(|item| (item.rule, item.line))
        .collect();
    found.sort();
    found
}

fn finding(rule: &str, line: usize) -> (String, Option<usize>) {
    (rule.to_owned(), Some(line))
}

#[test]
fn an_exception_lifts_only_the_rule_it_names_on_the_line_below() {
    let source = "fn read(text: &str) -> u32 {
    // DECISION: D042 panic-path
    format!(\"{} {}\", text, text).parse().unwrap()
}
// DECISION: D123
fn lowered(baseline: &Counts, current: &Counts) -> bool {
    baseline.len() > current.len()
}
fn first(items: &[u32]) -> u32 {
    // INVARIANT: I001
    items.first().copied().unwrap()
}
fn trace(value: u32) -> u32 {
    // DECISION: D042 debug-artifact
    dbg!(value)
}
";
    assert_eq!(
        found(source),
        [
            finding("comment-reference", 14),
            finding("debug-artifact", 15),
            finding("panic-path", 11),
            finding("positional-format", 3),
            finding("same-type-parameters", 6),
        ]
    );
}

#[test]
fn an_exception_whose_line_was_fixed_is_reported_by_the_comment_rule() {
    let excepted = "fn first(items: &[u32]) -> u32 {
    // DECISION: D042 panic-path
    items.first().copied().unwrap()
}
";
    assert!(found(excepted).is_empty());
    let fixed = excepted.replace(".unwrap()", ".unwrap_or_default()");
    assert_eq!(found(&fixed), [finding("comment-reference", 2)]);
}
