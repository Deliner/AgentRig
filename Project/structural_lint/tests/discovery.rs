use super::consumer::{check, consumer, write};
use crate::shared::testing::{Run, hermetic, repository, run, worker};
use crate::structural_lint::catalog;
use serde_json::{Value, json};
use std::{collections::BTreeMap, env, fs, path::Path};

const TEST_CODE_EXCLUDED: [&str; 38] = [
    "panic-path",
    "debug-assertion",
    "wildcard-import",
    "tuple-as-record",
    "eager-fallback",
    "discarded-result",
    "disabled-raii",
    "global-state",
    "super-super-reach",
    "unwind-justification",
    "guard-across-await",
    "blocking-in-async",
    "ord-eq-mismatch",
    "secret-debug",
    "unchecked-deserialize",
    "implicit-order",
    "unit-lock",
    "unlabeled-exit",
    "length-subtraction",
    "embedded-text",
    "test-placement",
    "same-type-parameters",
    "same-type-positions",
    "primitive-alias",
    "ambiguous-option",
    "partial-impl",
    "constant-default",
    "open-checked-field",
    "wrapper-deref",
    "primitive-from",
    "paired-calls",
    "variant-equality",
    "range-chain",
    "positional-format",
    "crossed-field",
    "positional-flag",
    "exposed-guard",
    "test-substitution",
];
const EXAMPLES: [&str; 5] = [
    "nonblank-lines",
    "directory-entries",
    "named-if-condition",
    "function-lines",
    "parameter-count",
];
const INVALID: [&[&str]; 4] = [
    &["lint-rule"],
    &["lint-rule", "unknown"],
    &["lint-rule", "function-lines", "--typo"],
    &["lint-rules", "extra"],
];

fn discover(args: &[&str]) -> Run {
    run(
        hermetic(worker()).args(args).current_dir(env::temp_dir()),
        "",
    )
}

fn described(args: &[&str]) -> String {
    let result = discover(args);
    assert_eq!(result.code, Some(0), "{args:?}: {}", result.output());
    result.stdout
}

fn rules() -> Vec<Value> {
    serde_json::from_value(catalog::catalog()).unwrap()
}

fn copy_skill(root: &Path, skill: &Value) {
    let skill = skill.as_str().unwrap();
    write(
        root,
        skill,
        &fs::read_to_string(repository().join(skill)).unwrap(),
    );
}

fn example_validates(kind: &str) {
    let example = described(&["lint-rule", kind, "--example"]);
    let (_temp, root) = consumer(&example);
    let config: Value = serde_yaml_ng::from_str(&example).unwrap();
    let rule = &config["rules"][0];
    for skill in [
        &config["config_skill"],
        &rule["warning_skill"],
        &rule["error_skill"],
    ] {
        copy_skill(&root, skill);
    }
    assert_eq!(check(&root), (Some(0), Vec::new()), "{kind}");
}

fn description_names_its_skill(kind: &str) {
    let item: Value = serde_json::from_str(&described(&["lint-rule", kind, "--json"])).unwrap();
    assert_eq!(item["kind"], kind);
    let skill = item["skill"].as_str().unwrap();
    let path = repository()
        .join(".agents/skills")
        .join(skill)
        .join("SKILL.md");
    assert!(path.is_file(), "{kind}: {skill}");
    let policy = kind == "named-if-condition";
    let parameters = if policy { "policy" } else { "thresholds" };
    assert_eq!(item["parameters"]["type"], parameters, "{kind}");
    let human = described(&["lint-rule", kind]);
    assert!(human.contains(item["metric"].as_str().unwrap()), "{human}");
    assert!(human.contains("--example"), "{human}");
}

#[test]
fn catalog_reports_handler_extensions() {
    let rules = rules();
    for kind in ["named-if-condition", "function-lines", "parameter-count"] {
        let rule = rules.iter().find(|item| item["kind"] == kind).unwrap();
        assert_eq!(rule["target"], "file", "{kind}");
        let handlers = json!({"rust": [".rs"], "python": [".py", ".pyi"]});
        assert_eq!(rule["handlers"], handlers, "{kind}");
        let extensions = rule["extensions"].as_array().unwrap().iter();
        let mut extensions: Vec<_> = extensions.map(|item| item.as_str().unwrap()).collect();
        extensions.sort();
        assert_eq!(extensions, [".py", ".pyi", ".rs"], "{kind}");
    }
}

#[test]
fn catalog_examples_validate() {
    for kind in EXAMPLES {
        example_validates(kind);
        description_names_its_skill(kind);
    }
}

#[test]
fn rule_examples_ship_excludes_only_where_decided() {
    let rules = rules();
    let excluding = rules.iter().map(|item| &item["defaults"]);
    let shipped: BTreeMap<_, _> = excluding
        .filter_map(|rule| Some((rule["id"].as_str()?, rule.get("exclude")?.clone())))
        .collect();
    let tests = json!(["**/tests/**", "**/tests.rs"]);
    let mut expected = BTreeMap::from(TEST_CODE_EXCLUDED.map(|id| (id, tests.clone())));
    let locks = json!(["**/Cargo.lock", "**/package-lock.json", "**/uv.lock"]);
    expected.insert("file-size", locks);
    assert_eq!(shipped, expected);
}

#[test]
fn discovery_rejects_invalid_arguments() {
    for args in INVALID {
        let result = discover(args);
        assert_eq!(result.code, Some(2), "{args:?}: {}", result.output());
        assert!(!result.stderr.is_empty(), "{args:?}");
    }
}
