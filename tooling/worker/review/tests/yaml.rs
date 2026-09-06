use review_runner::config::yaml;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Config {
    name: String,
    enabled: bool,
    thresholds: BTreeMap<String, u64>,
}

#[test]
fn typed_yaml_roundtrip_preserves_literals_and_nested_settings() {
    let source =
        "name: '&literal *literal !literal'\nenabled: true\nthresholds:\n  functions: 40\n";
    let config: Config = yaml::decode(source).unwrap();
    assert_eq!(config.name, "&literal *literal !literal");
    assert_eq!(config.thresholds["functions"], 40);
    let encoded = yaml::encode(&config).unwrap();
    assert_eq!(yaml::decode::<Config>(&encoded).unwrap(), config);
}

#[test]
fn duplicate_keys_and_yaml_composition_are_rejected() {
    for source in [
        "name: first\nname: second",
        "nested: {same: 1, same: 2}",
        "name: &anchor value",
        "name: *anchor",
        "name: !custom value",
        "name: !!str value",
        "nested: {<<: {name: value}}",
        "? [one, two]\n: value",
        "1: value",
        "number: .inf",
        "number: .nan",
        "---\na: 1\n---\nb: 2",
        "invalid: [",
    ] {
        assert!(
            yaml::decode::<serde_json::Value>(source).is_err(),
            "{source}"
        );
    }
}

#[test]
fn schema_errors_do_not_coerce_scalars_and_identify_the_field() {
    let source = "name: valid\nenabled: true\nthresholds: {functions: 40}\n";
    for (from, to, field) in [
        ("valid", "12", "name"),
        ("true", "'true'", "enabled"),
        ("true", "yes", "enabled"),
        ("40", "'40'", "thresholds.functions"),
        ("40", "-1", "thresholds.functions"),
        ("40", "1.5", "thresholds.functions"),
        ("name:", "unknown:", "unknown"),
    ] {
        let error = yaml::decode::<Config>(&source.replace(from, to)).unwrap_err();
        assert!(error.to_string().contains(field), "{error}");
    }
}

#[test]
fn file_errors_identify_source_and_do_not_read_toml_as_configuration() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("settings.yaml");
    fs::write(
        &path,
        "name='old'\nenabled=true\n[thresholds]\nfunctions=40",
    )
    .unwrap();
    let error = yaml::read::<Config>(&path).unwrap_err();
    assert!(format!("{error:#}").contains("settings.yaml"));
    fs::write(
        &path,
        "name: ok\nenabled: false\nthresholds: {functions: invalid}",
    )
    .unwrap();
    let error = format!("{:#}", yaml::read::<Config>(&path).unwrap_err());
    assert!(error.contains("settings.yaml") && error.contains("thresholds.functions"));
}
