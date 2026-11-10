from pathlib import Path

import pytest

from Project.structural_lint.architecture.tests.test_architecture import consumer
from Project.structural_lint.testing.consumer import lint


@pytest.mark.parametrize(
    "case",
    [
        (
            "py",
            'from importlib import import_module as load\nvalue = load("b.api").value\n',
            "python: [importlib]",
        ),
        (
            "py",
            'import importlib as loader\nvalue = loader.import_module("b.api").value\n',
            "python: [importlib]",
        ),
        ("py", 'load = __import__\nvalue = load("b.api")\n', "python: []"),
        ("rs", "#[generated]\npub fn run() -> i32 { 7 }\n", "rust: [std]"),
        (
            "rs",
            "#[derive(Generated)]\npub struct Value;\npub fn run() -> i32 { 7 }\n",
            "rust: [std]",
        ),
        (
            "rs",
            '#![cfg_attr(feature="generated", generated)]\npub fn run() -> i32 { 7 }\n',
            "rust: [std]",
        ),
    ],
)
def test_unanalyzed_loaders_and_attributes_cannot_pass(
    worker: Path, tmp_path: Path, case: tuple[str, str, str]
) -> None:
    language, source, external = case
    path, _ = consumer(tmp_path, language)
    path.write_text(source)
    policy = tmp_path / "lint.yaml"
    policy.write_text(
        policy.read_text().replace(
            "      python_root: src\n",
            f"      python_root: src\n      external:\n        {external}\n",
        )
    )
    code, findings = lint(worker, tmp_path)
    assert code == 1, findings
    assert any("incomplete" in item["message"] for item in findings)


def test_nonexpanding_rust_metadata_remains_supported(worker: Path, tmp_path: Path) -> None:
    path, _ = consumer(tmp_path, "rs")
    path.write_text('#![allow(dead_code)]\n#[inline]\n#[doc = "API"]\n' + path.read_text())
    assert lint(worker, tmp_path) == (0, [])


# DECISION: D141
@pytest.mark.parametrize(
    "case",
    [
        ("py", "src/a/api.py", 'load = __import__\nvalue = load("b.api")\n'),
        ("py", "src/a/api.py", "from b.missing import value\n"),
        ("rs", "src/lib.rs", "pub mod a; pub mod b; pub mod missing;\n"),
    ],
)
def test_incomplete_analysis_does_not_call_a_declared_external_unused(
    worker: Path, tmp_path: Path, case: tuple[str, str, str]
) -> None:
    language, path, source = case
    consumer(tmp_path, language)
    (tmp_path / path).write_text(source)
    key = {"py": "python", "rs": "rust"}[language]
    policy = tmp_path / "lint.yaml"
    policy.write_text(
        policy.read_text().replace(
            "      python_root: src\n",
            f"      python_root: src\n      external:\n        {key}: [unused_dep]\n",
        )
    )
    code, findings = lint(worker, tmp_path)
    assert code == 1, findings
    assert any("incomplete" in item["message"] for item in findings), findings
    assert all("unused_dep" not in item["message"] for item in findings), findings
