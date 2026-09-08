import json
import subprocess
from pathlib import Path

import pytest

from tooling.worker.src.scaffold.testing.consumer import file_contents, invoke, update_config


@pytest.mark.parametrize("case", ["invalid-glob", "file-parent", "generated-parent"])
def test_init_rejects_invalid_layout_before_writing(
    worker: Path, tmp_path: Path, case: str
) -> None:
    args = ["--source", "src/["]
    existing_file_parent = case == "file-parent"
    generated_file_parent = case == "generated-parent"
    if existing_file_parent:
        (tmp_path / ".codex").write_text("user data")
        args = []
    elif generated_file_parent:
        args = ["--skills", ".agentrig/bin/agentrig"]
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "init", *args)
    assert result.returncode == 2
    assert file_contents(tmp_path) == before


@pytest.mark.parametrize("language", ["python", "rust"])
def test_custom_service_executes_installed_adapters(
    worker: Path, tmp_path: Path, language: str
) -> None:
    service = "rig space's $cash"
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    result = invoke(worker, tmp_path, "init", "--language", language, "--service", service)
    assert result.returncode == 0, result.stderr
    assert not (tmp_path / ".agentrig").exists()
    assert not (tmp_path / ".worker").exists()
    installed = tmp_path / service / "bin/agentrig"
    assert invoke(installed, tmp_path, "doctor").returncode == 0
    checked = subprocess.run(
        ["just", "config-check"], cwd=tmp_path, capture_output=True, text=True, check=False
    )
    assert checked.returncode == 0, checked.stderr
    hook = json.loads((tmp_path / ".codex/hooks.json").read_text())
    command = hook["hooks"]["SessionStart"][0]["hooks"][0]["command"]
    started = subprocess.run(
        ["sh", "-c", command],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
        input=json.dumps({"hook_event_name": "SessionStart", "session_id": "custom-service"}),
    )
    assert started.returncode == 0, started.stderr
    assert "complexity" in started.stdout
    blocked = subprocess.run(
        [str(tmp_path / service / "hooks/pre-commit")],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
    )
    assert blocked.returncode != 0
    assert "feature" in blocked.stderr
    assert (tmp_path / service / "manifest.json").is_file()
    assert (tmp_path / service / "runtime/reminders").is_dir()


@pytest.mark.parametrize("service", ["../escape", "bad\npath", "{{injection}}"])
def test_invalid_service_leaves_consumer_untouched(
    worker: Path, tmp_path: Path, service: str
) -> None:
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "init", "--service", service)
    assert result.returncode == 2
    assert file_contents(tmp_path) == before


def test_setup_cannot_silently_relocate_an_installation(worker: Path, tmp_path: Path) -> None:
    assert invoke(worker, tmp_path, "init").returncode == 0
    update_config(tmp_path / "agentrig.yaml", paths={"service": "replacement"})
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "setup")
    assert result.returncode == 2
    assert "setup conflict" in result.stderr
    assert file_contents(tmp_path) == before
