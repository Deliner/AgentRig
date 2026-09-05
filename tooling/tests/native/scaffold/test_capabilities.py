import json
from pathlib import Path

from support import CONFIG, invoke, project
from test_review import resources


def test_review_uses_project_capability_configuration(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, CONFIG + '\n[capabilities.review]\nconfig="config.toml"\n')
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
    (tmp_path / "lint.toml").unlink()
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
