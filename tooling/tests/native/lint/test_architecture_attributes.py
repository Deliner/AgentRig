from pathlib import Path

import pytest

from tooling.tests.native.lint.test_architecture import consumer, contract
from tooling.tests.native.lint.test_architecture_behavior import parity, run_consumer


@pytest.mark.parametrize("case", ["callback", "label", "unknown", "derive", "alias", "test"])
def test_rust_attributes_preserve_dependency_evidence(
    worker: Path, tmp_path: Path, case: str
) -> None:
    source, _ = consumer(tmp_path, "rs")
    samples = {
        "callback": '#[derive(serde::Deserialize)] pub struct Value { #[serde(default = "crate::b::value")] x: i32 }',
        "label": '#[derive(serde::Deserialize)] pub struct Value { #[serde(rename = "crate::missing::value")] x: i32 }',
        "unknown": '#[derive(serde::Deserialize)] #[serde(unknown = "value")] pub struct Value;',
        "derive": "#[derive(custom::Debug)] pub struct Value;",
        "alias": "use custom::Debug; #[derive(Debug)] pub struct Value;",
        "test": "#[cfg(test)] mod checks { pub fn run() -> i32 { crate::b::value() } }",
    }
    source.write_text(samples[case])
    policy = tmp_path / "lint.yaml"
    policy.write_text(
        policy.read_text().replace(
            "      rust_roots:",
            "      external:\n        rust: [std, serde, custom]\n      rust_roots:",
        )
    )
    code, findings = parity(worker, tmp_path)
    unsupported = case in {"unknown", "derive", "alias"}
    if unsupported:
        assert code == 1 and any("incomplete" in item["message"] for item in findings), findings
    else:
        assert code == 0, findings
        contract(tmp_path / "src/a")
        code, findings = parity(worker, tmp_path)
        edge = case in {"callback", "test"}
        assert code == int(edge), findings
        assert all("forbidden" in item["message"] for item in findings)


def test_standard_derive_preserves_compiled_consumer(worker: Path, tmp_path: Path) -> None:
    source, _ = consumer(tmp_path, "rs")
    source.write_text(
        '#[derive(Debug, Clone, Default)] pub struct Value; pub fn run() -> i32 { let _ = format!("{:?}", Value); 7 }'
    )
    entry = tmp_path / "src/lib.rs"
    entry.write_text(entry.read_text() + "fn main() { std::process::exit(a::run()); }")
    assert run_consumer(tmp_path, "rs") == "7"
    assert parity(worker, tmp_path) == (0, [])
