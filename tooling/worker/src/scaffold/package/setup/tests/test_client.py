import json
import subprocess
import tomllib
from pathlib import Path

import pytest

from tooling.worker.src.scaffold.testing.consumer import file_contents, invoke, update_config


@pytest.mark.parametrize("frontend", ["codex", "claude-code"])
@pytest.mark.parametrize("custom_endpoint", [True, False])
def test_project_model_and_api_references(
    worker: Path, tmp_path: Path, frontend: str, custom_endpoint: bool
) -> None:
    assert invoke(worker, tmp_path, "init", "--frontend", frontend).returncode == 0
    api = {"key_env": "AGENT_TEST_KEY"}
    if custom_endpoint:
        api["base_url"] = "https://gateway.example/v1"
    agent = {
        "model": "test-model",
        "reasoning_effort": "high",
        "api": api,
    }
    update_config(tmp_path / "agentrig.yaml", agent=agent)
    before = file_contents(tmp_path)
    preview = invoke(worker, tmp_path, "setup", "--preview")
    assert preview.returncode == 0, preview.stderr
    assert json.loads(preview.stdout)["agent"] == agent
    assert file_contents(tmp_path) == before
    result = invoke(worker, tmp_path, "setup")
    assert result.returncode == 0, result.stderr
    verify_project_api(tmp_path, frontend, custom_endpoint)
    before = file_contents(tmp_path)
    assert invoke(worker, tmp_path, "setup").returncode == 0
    assert file_contents(tmp_path) == before
    update_config(tmp_path / "agentrig.yaml", agent={"model": "conflicting-model"})
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "setup")
    assert result.returncode == 2
    assert "setup conflict" in result.stderr and "model" in result.stderr
    assert file_contents(tmp_path) == before


def verify_project_api(root: Path, frontend: str, custom_endpoint: bool) -> None:
    codex = frontend == "codex"
    if codex:
        settings = tomllib.loads((root / ".codex/config.toml").read_text())
        assert settings["model"] == "test-model"
        assert settings["model_reasoning_effort"] == "high"
        provider = settings["model_providers"][settings["model_provider"]]
        assert provider["env_key"] == "AGENT_TEST_KEY"
        endpoint = "https://gateway.example/v1" if custom_endpoint else "https://api.openai.com/v1"
        assert provider["base_url"] == endpoint
        assert provider["wire_api"] == "responses"
        assert provider["requires_openai_auth"] is False
        return
    settings = json.loads((root / ".claude/settings.json").read_text())
    assert settings["model"] == "test-model"
    assert settings["effortLevel"] == "high"
    claude_endpoint = "https://gateway.example/v1" if custom_endpoint else None
    assert settings.get("env", {}).get("ANTHROPIC_BASE_URL") == claude_endpoint
    helper = ["sh", "-c", settings["apiKeyHelper"]]
    key = "fake-'quoted-$(never-execute)-key"
    result = subprocess.run(
        helper, env={"AGENT_TEST_KEY": key}, capture_output=True, text=True, check=False
    )
    assert result.returncode == 0 and result.stdout == key
    missing = subprocess.run(helper, env={}, capture_output=True, text=True, check=False)
    assert missing.returncode != 0 and not missing.stdout
    assert "missing agent.api.key_env reference" in missing.stderr
    assert key not in (root / ".claude/settings.json").read_text()


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
