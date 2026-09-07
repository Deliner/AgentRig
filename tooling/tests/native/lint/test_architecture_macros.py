from pathlib import Path

import pytest
from test_architecture import consumer, contract
from test_architecture_behavior import parity


@pytest.mark.parametrize(
    "case",
    [
        "assert",
        "nested",
        "module",
        "anyhow",
        "json",
        "unknown",
        "alias",
        "local",
        "wildcard",
        "inner",
    ],
)
def test_macro_dependencies_and_provenance(worker: Path, tmp_path: Path, case: str) -> None:
    source, _ = consumer(tmp_path, "rs")
    sources = {
        "assert": "pub fn run() -> i32 { assert!(crate::b::value() == 7); 7 }",
        "nested": 'pub fn run() -> i32 { assert_eq!(format!("{}", crate::b::value()), "7"); 7 }',
        "module": 'use crate::b as format; pub fn run() -> i32 { assert_eq!(format!("{}", format::value()), "7"); 7 }',
        "anyhow": "use anyhow::ensure; pub fn run() { ensure!(crate::b::value() == 7); }",
        "json": 'use serde_json::json; pub fn run() { let _ = json!({"key": [crate::b::value()]}); }',
        "unknown": "pub fn run() { generate!(crate::b::value()); }",
        "alias": "use custom::assert; pub fn run() { assert!(true); }",
        "local": "pub fn run() { use custom::assert; assert!(true); }",
        "wildcard": "use custom::*; pub fn run() { assert!(true); }",
        "inner": "pub fn run() { assert!({ use custom::assert; assert!(true); true }); }",
    }
    source.write_text(sources[case])
    config = tmp_path / "lint.yaml"
    config.write_text(
        config.read_text().replace(
            "      rust_roots:",
            "      external:\n        rust: [std, anyhow, serde_json, custom]\n      rust_roots:",
        )
    )
    code, findings = parity(worker, tmp_path)
    unknown = case in {"unknown", "alias", "local", "wildcard", "inner"}
    if unknown:
        assert code == 1 and any("incomplete" in item["message"] for item in findings), findings
    else:
        assert code == 0, findings
        contract(tmp_path / "src/a")
        code, findings = parity(worker, tmp_path)
        assert code == 1 and any("forbidden dependency" in item["message"] for item in findings), (
            findings
        )
