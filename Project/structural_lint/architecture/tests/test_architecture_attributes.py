from pathlib import Path

import pytest

from Project.structural_lint.architecture.tests.test_architecture import back_edge, consumer
from Project.structural_lint.architecture.tests.test_architecture_behavior import (
    parity,
    run_consumer,
)

ATTR_EXTERNALS = {
    "callback": "[std, serde]",
    "label": "[std, serde]",
    "test": "[std]",
    "platform": "[std]",
    "feature": "[std]",
    # DECISION: D136
    "export": "[std]",
    "bindgen": "[std, wasm_bindgen]",
}


@pytest.mark.parametrize(
    "case",
    [
        "callback",
        "label",
        "unknown",
        "derive",
        "alias",
        "test",
        "platform",
        "feature",
        "export",
        "bindgen",
        "local",
    ],
)
def test_rust_attributes_preserve_dependency_evidence(
    worker: Path, tmp_path: Path, case: str
) -> None:
    source, target = consumer(tmp_path, "rs")
    samples = {
        "callback": '#[derive(serde::Deserialize)] pub struct Value { #[serde(default = "crate::b::value")] x: i32 }',
        "label": '#[derive(serde::Deserialize)] pub struct Value { #[serde(rename = "crate::missing::value")] x: i32 }',
        "unknown": '#[derive(serde::Deserialize)] #[serde(unknown = "value")] pub struct Value;',
        "derive": "#[derive(custom::Debug)] pub struct Value;",
        "alias": "use custom::Debug; #[derive(Debug)] pub struct Value;",
        "test": "#[cfg(test)] mod checks { pub fn run() -> i32 { crate::b::value() } }",
        "platform": '#[cfg(not(target_os = "none"))] mod checks { pub fn run() -> i32 { crate::b::value() } }',
        "feature": '#[cfg(feature = "extra")] mod checks { pub fn run() -> i32 { crate::b::value() } }',
        "export": '#[unsafe(no_mangle)] pub extern "C" fn run() -> i32 { crate::b::value() } #[export_name = "other"] pub fn other() {}',
        "bindgen": "use wasm_bindgen::prelude::*; #[wasm_bindgen] pub fn run() -> i32 { crate::b::value() }",
        "local": "#[crate::b::attribute] pub fn run() {}",
    }
    source.write_text(samples[case])
    declared = ATTR_EXTERNALS.get(case, "[std, serde, custom]")
    policy = tmp_path / "lint.yaml"
    policy.write_text(
        policy.read_text().replace(
            "      rust_roots:",
            f"      external:\n        rust: {declared}\n      rust_roots:",
        )
    )
    code, findings = parity(worker, tmp_path)
    unsupported = case in {"unknown", "derive", "alias", "local"}
    if unsupported:
        assert code == 1 and any("incomplete" in item["message"] for item in findings), findings
    else:
        assert code == 0, findings
        back_edge(target, "rs")
        code, findings = parity(worker, tmp_path)
        edge = case in {"callback", "test", "platform", "feature", "export", "bindgen"}
        assert code == int(edge), findings
        assert all("directory dependency cycle" in item["message"] for item in findings)


def test_standard_derive_preserves_compiled_consumer(worker: Path, tmp_path: Path) -> None:
    source, _ = consumer(tmp_path, "rs")
    source.write_text(
        '#[derive(Debug, Clone, Default)] pub struct Value; pub fn run() -> i32 { let _ = format!("{:?}", Value); 7 }'
    )
    entry = tmp_path / "src/lib.rs"
    entry.write_text(entry.read_text() + "fn main() { std::process::exit(a::run()); }")
    assert run_consumer(tmp_path, "rs") == "7"
    assert parity(worker, tmp_path) == (0, [])
