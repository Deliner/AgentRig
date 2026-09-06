import json
from pathlib import Path

from support import CONFIG, invoke, project
from test_review import resources


def test_review_uses_project_capability_configuration(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, CONFIG + '\n[capabilities.review]\nconfig="config.yaml"\n')
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
    source = CONFIG + "\n[capabilities]\nlint=false\n"
    project(tmp_path, source)
    (tmp_path / "lint.yaml").unlink()
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 0, result.stderr
    result = invoke(worker, tmp_path, "lint")
    assert result.returncode == 2
    assert "capabilities.lint is disabled" in result.stderr
    (tmp_path / "worker.toml").write_text(
        source + '\n[[checks]]\nid="lint"\nkind="lint"\nskill="guides/repair/SKILL.md"\n'
    )
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "lint check is configured" in result.stderr


def test_unknown_and_absent_capabilities_are_actionable(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    result = invoke(worker, tmp_path, "review", "config-check")
    assert result.returncode == 2
    assert "review is not enabled" in result.stderr
    (tmp_path / "worker.toml").write_text(CONFIG + "\n[capabilities]\nunknown=true\n")
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "unknown field `unknown`" in result.stderr


def test_installation_ships_review_resources(worker: Path, tmp_path: Path) -> None:
    result = invoke(worker, tmp_path, "init", "--review", "true", "--skills", "guides")
    assert result.returncode == 0, result.stderr
    installed = tmp_path / ".worker/bin/discipline-worker"
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
