# DECISION: D021
import json
from pathlib import Path
from typing import Any

import pytest
import yaml

from Project.shared.testing.consumer import invoke


@pytest.mark.parametrize(
    ("language", "rule"),
    [
        ("python", "named-if"),
        ("rust", "named-if"),
        ("python", "function-size"),
        ("rust", "function-size"),
        ("python", "parameters"),
        ("rust", "parameters"),
        ("python", "comments"),
        ("rust", "comments"),
    ],
)
# INVARIANT: I017
def test_strict_defaults_block_delivery(
    worker: Path, tmp_path: Path, language: str, rule: str
) -> None:
    assert invoke(worker, tmp_path, "init", "--language", language).returncode == 0
    source = tmp_path / "Project"
    source.mkdir()
    suffix = {"python": "py", "rust": "rs"}[language]
    path = source / f"example.{suffix}"
    invalid, boundary = examples(language)[rule]
    path.write_text(invalid)
    result = invoke(worker, tmp_path, "lint", "--json")
    assert result.returncode == 1
    findings = json.loads(result.stdout)
    assert any(item["rule"] == rule and item["level"] == "error" for item in findings)
    assert all(item["skill"].endswith("SKILL.md") for item in findings)
    result = invoke(worker, tmp_path, "check")
    assert result.returncode == 1
    assert "ERROR [lint]" in result.stderr
    path.write_text(boundary)
    assert invoke(worker, tmp_path, "lint").returncode == 0


def test_shipped_architecture_reports_contract_and_cycle(worker: Path, tmp_path: Path) -> None:
    assert invoke(worker, tmp_path, "init", "--language", "python").returncode == 0
    for name in ("alpha", "beta"):
        (tmp_path / "Project" / name).mkdir(parents=True)
    (tmp_path / "Project/alpha/api.py").write_text(
        "from Project.beta.api import value\n\n\ndef run():\n    return value()\n"
    )
    (tmp_path / "Project/beta/api.py").write_text("def value():\n    return 7\n")
    missing = architecture_findings(worker, tmp_path)
    assert any("missing" in item["message"] for item in missing), missing
    for name in ("alpha", "beta"):
        contract(tmp_path / "Project" / name)
    beta = tmp_path / "Project/beta/api.py"
    beta.write_text(beta.read_text() + "import Project.alpha.api\n")
    cycle = architecture_findings(worker, tmp_path)
    assert any("cycle" in item["message"] for item in cycle), cycle


def architecture_findings(worker: Path, root: Path) -> list[dict[str, Any]]:
    result = invoke(worker, root, "lint", "--json")
    assert result.returncode == 1, result.stdout + result.stderr
    findings = json.loads(result.stdout)
    assert all(item["skill"].endswith("refactor-large-directory/SKILL.md") for item in findings)
    selected = []
    for item in findings:
        architecture = item["rule"] == "architecture"
        if architecture:
            selected.append(item)
    return selected


def contract(directory: Path) -> None:
    files = {}
    for path in directory.iterdir():
        regular = path.is_file() and path.name != "architecture.yaml"
        if regular:
            files[path.name] = "source"
    (directory / "architecture.yaml").write_text(
        f"purpose: {directory.name}\n" + yaml.safe_dump({"files": files, "directories": {}})
    )


def examples(language: str) -> dict[str, tuple[str, str]]:
    python = language == "python"
    if python:
        return {
            "named-if": ("if True:\n    pass\n", "ready = True\nif ready:\n    pass\n"),
            "function-size": (
                "def example():\n" + "    value = 0\n" * 40,
                "def example():\n" + "    value = 0\n" * 39,
            ),
            "parameters": ("def example(a, b, c, d, e): pass\n", "def example(a, b, c, d): pass\n"),
            "comments": ("# note\nvalue = 0\n", "# DECISION: D001\nvalue = 0\n"),
        }
    return {
        "named-if": (
            "fn example() { if true {} }\n",
            "fn example() { let ready = true; if ready {} }\n",
        ),
        "function-size": (
            "fn example() {\n" + "    let value = 0;\n" * 39 + "}\n",
            "fn example() {\n" + "    let value = 0;\n" * 38 + "}\n",
        ),
        "parameters": (
            "fn example(a:i32,b:i32,c:i32,d:i32,e:i32) {}\n",
            "fn example(a:i32,b:i32,c:i32,d:i32) {}\n",
        ),
        "comments": ("/// note\nfn example() {}\n", "// INVARIANT: I001\nfn example() {}\n"),
    }
