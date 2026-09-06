import json
from pathlib import Path

import pytest
from support import CONFIG, invoke, project
from test_review import resources


def test_review_uses_project_capability_configuration(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, CONFIG + '\ncapabilities:\n  review:\n    config: "config.yaml"\n')
    resources(tmp_path)
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 0, result.stderr
    result = invoke(worker, tmp_path, "review", "config-check")
    assert result.returncode == 0, result.stderr
    messages = [
        dict(jsonrpc="2.0", id=1, method="initialize", params=dict(protocolVersion="2025-11-25")),
        dict(jsonrpc="2.0", id=2, method="tools/list"),
    ]
    result = invoke(
        worker,
        tmp_path,
        "review",
        "mcp",
        input="".join(json.dumps(message) + "\n" for message in messages),
    )
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout.splitlines()[1])["result"]["tools"][0]["name"] == "review_code"
    (tmp_path / "prompt.md").unlink()
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "capabilities.review.config" in result.stderr


def test_disabled_lint_requires_consistent_gate(worker: Path, tmp_path: Path) -> None:
    source = CONFIG + "\ncapabilities:\n  lint: false\n"
    project(tmp_path, source)
    (tmp_path / "lint.yaml").unlink()
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 0, result.stderr
    result = invoke(worker, tmp_path, "lint")
    assert result.returncode == 2
    assert "capabilities.lint is disabled" in result.stderr
    (tmp_path / "agentrig.yaml").write_text(
        source + '\nchecks:\n- id: "lint"\n  kind: "lint"\n  skill: "guides/repair/SKILL.md"\n'
    )
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "lint check is configured" in result.stderr


def test_unknown_and_absent_capabilities_are_actionable(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    result = invoke(worker, tmp_path, "review", "config-check")
    assert result.returncode == 2
    assert "review is not enabled" in result.stderr
    (tmp_path / "agentrig.yaml").write_text(CONFIG + "\ncapabilities:\n  unknown: true\n")
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "unknown field `unknown`" in result.stderr


def test_installation_ships_review_resources(worker: Path, tmp_path: Path) -> None:
    result = invoke(worker, tmp_path, "init", "--review", "true", "--skills", "guides")
    assert result.returncode == 0, result.stderr
    installed = tmp_path / ".worker/bin/agentrig"
    result = invoke(installed, tmp_path, "review", "config-check")
    assert result.returncode == 0, result.stderr
    messages = [
        dict(jsonrpc="2.0", id=1, method="initialize", params=dict(protocolVersion="2025-11-25")),
        dict(jsonrpc="2.0", id=2, method="tools/list"),
    ]
    result = invoke(
        installed,
        tmp_path,
        "review",
        "mcp",
        input="".join(json.dumps(message) + "\n" for message in messages),
    )
    assert result.returncode == 0, result.stderr
    tools = json.loads(result.stdout.splitlines()[1])["result"]["tools"]
    assert {tool["name"] for tool in tools} == {"review_code", "review_research"}
    receipt = json.loads((tmp_path / ".worker/manifest.json").read_text())["files"]
    assert receipt["guides/review-project/SKILL.md"]["ownership"] == "editable"
    assert receipt[".worker/review/config/review.yaml"]["ownership"] == "configuration"
    assert receipt[".worker/review/prompts/correctness.md"]["ownership"] == "editable"
    assert receipt["AGENTS.md"]["ownership"] == "editable"


@pytest.mark.parametrize(
    "replacement", ['version: "1"', "version: 1\nversion: 1", "version: &schema 1"]
)
def test_root_yaml_is_strict(worker: Path, tmp_path: Path, replacement: str) -> None:
    project(tmp_path, CONFIG.replace("version: 1", replacement, 1))
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "agentrig.yaml" in result.stderr and "ACTION:" in result.stderr


def test_legacy_root_requires_explicit_migration(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    (tmp_path / "agentrig.yaml").unlink()
    (tmp_path / "worker.toml").write_text('version = 1\nruntime = "0.2.0"\n')
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "explicit upgrade" in result.stderr and "no format fallback" in result.stderr
