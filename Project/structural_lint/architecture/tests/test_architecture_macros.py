from pathlib import Path

import pytest

from Project.structural_lint.architecture.tests.test_architecture import consumer
from Project.structural_lint.architecture.tests.test_architecture_behavior import (
    assert_back_edge_forms_cycle,
    parity,
)

MACRO_EXTERNALS = {
    "assert": "[std]",
    "nested": "[std]",
    "module": "[std]",
    "anyhow": "[std, anyhow]",
    "json": "[std, serde_json]",
    "alias": "[std, custom]",
    "wildcard": "[std, custom]",
    "definition": "[std]",
    "generated": "[std]",
}


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
        "definition",
        "generated",
    ],
)
def test_macro_dependencies_and_provenance(worker: Path, tmp_path: Path, case: str) -> None:
    source, target = consumer(tmp_path, "rs")
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
        "definition": "macro_rules! value { ($x:expr) => { $crate::b::value() + $x }; } pub fn run() -> i32 { value!(0) }",
        "generated": "macro_rules! make { ($name:ident) => { pub struct $name; impl $name { pub fn get() -> i32 { crate::b::value() } } }; } make!(Made); pub fn run() -> i32 { self::Made::get() }",
    }
    source.write_text(sources[case])
    declared = MACRO_EXTERNALS.get(case, "[std, anyhow, serde_json, custom]")
    config = tmp_path / "lint.yaml"
    config.write_text(
        config.read_text().replace(
            "      rust_roots:",
            f"      external:\n        rust: {declared}\n      rust_roots:",
        )
    )
    code, findings = parity(worker, tmp_path)
    unknown = case in {"unknown", "local", "inner"}
    # DECISION: D136
    external = case in {"alias", "wildcard"}
    if unknown:
        assert code == 1 and any("incomplete" in item["message"] for item in findings), findings
    elif external:
        assert (code, findings) == (0, [])
    else:
        assert code == 0, findings
        assert_back_edge_forms_cycle(worker, tmp_path, target)
