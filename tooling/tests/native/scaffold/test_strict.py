# DECISION: D021
import json
from pathlib import Path

import pytest

from tooling.tests.native.scaffold.support import file_contents, invoke, update_config


@pytest.mark.parametrize("frontend", ["codex", "claude-code"])
@pytest.mark.parametrize(
    "agent",
    [
        {"model": " "},
        {"reasoning_effort": "max"},
        {"api": {"key_env": "KEY; touch injected"}},
        {"api": {"key_env": "KEY", "base_url": "not-a-url"}},
        {"api": {"key": "literal-secret"}},
    ],
)
def test_invalid_project_agent_preserves_files(
    worker: Path, tmp_path: Path, frontend: str, agent: dict[str, object]
) -> None:
    assert invoke(worker, tmp_path, "init", "--frontend", frontend).returncode == 0
    update_config(tmp_path / "agentrig.yaml", agent=agent)
    before = file_contents(tmp_path)
    for args in [("setup", "--preview"), ("setup",), ("config-check",)]:
        result = invoke(worker, tmp_path, *args)
        assert result.returncode == 2, result.stdout + result.stderr
        assert "agent" in result.stderr or "unknown field" in result.stderr
        assert file_contents(tmp_path) == before


@pytest.mark.parametrize(
    ("language", "rule"),
    [
        ("python", "named-if"),
        ("rust", "named-if"),
        ("python", "function-size"),
        ("rust", "function-size"),
        ("python", "parameters"),
        ("rust", "parameters"),
    ],
)
# INVARIANT: I017
def test_strict_defaults_block_delivery(
    worker: Path, tmp_path: Path, language: str, rule: str
) -> None:
    assert invoke(worker, tmp_path, "init", "--language", language).returncode == 0
    source = tmp_path / "src"
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
    }
