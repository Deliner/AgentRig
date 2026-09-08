from pathlib import Path

import pytest

from tooling.tests.native.lint.test_architecture import consumer, contract
from tooling.tests.native.test_lint import lint


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
        ("js", 'const load = require;\nmodule.exports = load("../b/api.js");\n', "javascript: []"),
        (
            "js",
            'import {createRequire} from "node:module";\nconst load = createRequire(import.meta.url);\nload("../b/api.js");\n',
            "javascript: ['node:module']",
        ),
        (
            "ts",
            'import {createRequire as factory} from "node:module";\nconst load = factory(import.meta.url);\nload("../b/api.js");\n',
            "javascript: ['node:module']",
        ),
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
    contract(tmp_path / "src/a")
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
