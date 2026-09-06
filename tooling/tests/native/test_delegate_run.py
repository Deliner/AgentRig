import hashlib
import json
import os
import subprocess
import time
from pathlib import Path
from typing import Any

import pytest


def consumer(worker: Path, root: Path, monkeypatch: pytest.MonkeyPatch, script: str) -> None:
    result = subprocess.run(
        [str(worker), "init", "--root", str(root)], capture_output=True, text=True, check=False
    )
    assert result.returncode == 0, result.stderr
    binary = root / "fixture/codex"
    binary.parent.mkdir()
    binary.write_text("#!/bin/sh\nset -eu\n" + script)
    binary.chmod(0o755)
    binary.with_name("codex-code-mode-host").write_text("")
    monkeypatch.setenv("DELEGATE_CODEX_BIN", str(binary))
    monkeypatch.setenv("DELEGATE_FAKE_KEY", "unused-fixture-key")
    monkeypatch.setenv("WORKER_OWNER", "delegate-fixture")
    (root / "prompt.md").write_text("Complete the task.\n")
    (root / "delegate.yaml").write_text(CONFIG)
    request = {
        "profile": "fixture",
        "task": "Generate an artifact",
        "contract": {
            "result_schema": {
                "type": "object",
                "required": ["ok"],
                "properties": {"ok": {"const": True}},
            },
            "artifacts": {"asset.txt": 16},
        },
    }
    (root / "request.json").write_text(json.dumps(request))


CONFIG = """schema_version: 1
profiles:
  fixture:
    frontend: "codex"
    model: "fixture"
    reasoning_effort: "high"
    mode: "artifacts"
    prompt: "prompt.md"
    visible_paths: ["src/**"]
    timeout_seconds: 2
    programs:
      sleep: "/usr/bin/sleep"
    credentials:
      env:
        OPENAI_API_KEY: "DELEGATE_FAKE_KEY"
"""


def call(worker: Path, root: Path, *args: str) -> dict[str, Any]:
    result = subprocess.run(
        [str(worker), "delegate", "--root", str(root), *args],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    value: dict[str, Any] = json.loads(result.stdout)
    return value


def terminal(worker: Path, root: Path, identifier: str) -> dict[str, Any]:
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        value = call(worker, root, "result", identifier)
        done = value["job"]["state"] in {"completed", "cancelled", "interrupted"}
        if done:
            return value
        time.sleep(0.05)
    raise AssertionError("delegate did not finish")


def test_async_artifact_result_survives_reconnect_and_cleanup(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    consumer(
        worker,
        tmp_path,
        monkeypatch,
        "printf artifact > /work/asset.txt\nprintf '{\"ok\":true}' > /work/result.json\n",
    )
    identifier = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    result = terminal(worker, tmp_path, identifier)
    assert result["job"]["exit_code"] == 0, result
    assert result["outcome"] == "PASS", result
    assert result["report"]["status"] == "PASS", result
    assert result["report"]["cleanup_errors"] == []
    assert call(worker, tmp_path, "result", identifier)["report"] == result["report"]
    directory = tmp_path / ".agentrig/runtime/jobs" / identifier
    assert (directory / "artifacts/asset.txt").read_text() == "artifact"
    assert not (directory / "input").exists()
    assert not (directory / "private").exists()
    receipt = json.loads((directory / "environment.json").read_text())
    assert result["environment"] == receipt
    assert receipt["files"]["programs/sleep"] == {
        "sha256": hashlib.sha256(Path("/usr/bin/sleep").read_bytes()).hexdigest(),
        "executable": True,
    }
    assert call(worker, tmp_path, "result", identifier)["environment"] == receipt


def test_timeout_is_a_persisted_error(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    consumer(worker, tmp_path, monkeypatch, "/tools/sleep 60\n")
    identifier = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    result = terminal(worker, tmp_path, identifier)
    assert result["report"]["status"] == "ERROR", result
    assert result["outcome"] == "ERROR", result
    assert "124" in result["report"]["error"]
    assert result["report"]["cleanup_errors"] == []


def test_cancel_recovers_report_and_cleans_private_files(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    consumer(worker, tmp_path, monkeypatch, "/tools/sleep 60\n")
    identifier = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    result = call(worker, tmp_path, "cancel", identifier)
    assert result["job"]["state"] == "cancelled", result
    assert result["outcome"] == "CANCELLED", result
    assert result["report"]["status"] == "ERROR", result
    assert result["report"]["cleanup_errors"] == []
    assert not (tmp_path / ".agentrig/runtime/jobs" / identifier / "private").exists()


def test_profile_limits_reach_kernel_cgroup(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    consumer(worker, tmp_path, monkeypatch, "/tools/sleep 60\n")
    config = tmp_path / "delegate.yaml"
    config.write_text(
        config.read_text().replace(
            "timeout_seconds: 2",
            "timeout_seconds: 60\n    memory_bytes: 268435456\n    max_processes: 64",
        )
    )
    identifier = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    try:
        status = call(worker, tmp_path, "status", identifier)
        group = status["job"]["scope_observation"]["control_group"]
        directory = Path("/sys/fs/cgroup") / group.lstrip("/")
        assert (directory / "memory.max").read_text().strip() == "268435456"
        assert (directory / "pids.max").read_text().strip() == "64"
    finally:
        call(worker, tmp_path, "cancel", identifier)


def test_read_delegate_uses_explicit_read_only_inputs(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    script = 'read value < /inputs/value\ntest "$value" = sample\n! echo changed > /inputs/value\nprintf \'{"ok":true}\' > /work/result.json\n'
    consumer(worker, tmp_path, monkeypatch, script)
    config = tmp_path / "delegate.yaml"
    config.write_text(config.read_text().replace('mode: "artifacts"', 'mode: "read"'))
    source = tmp_path / "src/value.txt"
    source.parent.mkdir(exist_ok=True)
    source.write_text("sample\n")
    request_file = tmp_path / "request.json"
    request = json.loads(request_file.read_text())
    request["inputs"] = {"value": "src/value.txt"}
    request["contract"]["artifacts"] = {}
    request_file.write_text(json.dumps(request))
    identifier = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    result = terminal(worker, tmp_path, identifier)
    assert result["outcome"] == "PASS", result
    assert source.read_text() == "sample\n"


def test_cleanup_failure_cannot_be_overall_pass_and_can_be_retried(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    script = "printf artifact > /work/asset.txt\nprintf '{\"ok\":true}' > /work/result.json\n/tools/chmod 000 /codex\n"
    consumer(worker, tmp_path, monkeypatch, script)
    config = tmp_path / "delegate.yaml"
    config.write_text(
        config.read_text().replace("    programs:", '    programs:\n      chmod: "/usr/bin/chmod"')
    )
    identifier = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    directory = tmp_path / ".agentrig/runtime/jobs" / identifier
    try:
        result = terminal(worker, tmp_path, identifier)
        assert result["outcome"] == "ERROR", result
        assert result["report"]["cleanup_errors"], result
        assert (directory / "artifacts/asset.txt").read_text() == "artifact"
    finally:
        (directory / "private/codex").chmod(0o700)
        result = call(worker, tmp_path, "result", identifier)
        assert result["report"]["cleanup_errors"] == [], result
        assert not (directory / "private").exists()


@pytest.mark.parametrize("cancel", [False, True])
def test_slow_launcher_survives_start_disconnect(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, cancel: bool
) -> None:
    consumer(
        worker,
        tmp_path,
        monkeypatch,
        "printf artifact > /work/asset.txt\nprintf '{\"ok\":true}' > /work/result.json\n",
    )
    launcher = tmp_path / "fixture/systemd-run"
    launcher.write_text('#!/bin/sh\n/bin/sleep 7\nexec /usr/bin/systemd-run "$@"\n')
    launcher.chmod(0o755)
    monkeypatch.setenv("PATH", str(launcher.parent) + os.pathsep + os.environ["PATH"])
    identifier = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    result = call(worker, tmp_path, "result", identifier)
    assert result["outcome"] == "RUNNING", result
    assert (tmp_path / ".agentrig/runtime/jobs" / identifier / "input").exists()
    if cancel:
        result = call(worker, tmp_path, "cancel", identifier)
        assert result["job"]["state"] == "stopping", result
        assert terminal(worker, tmp_path, identifier)["outcome"] == "CANCELLED"
        assert not (tmp_path / ".agentrig/runtime/jobs" / identifier / "artifacts").exists()
    else:
        assert terminal(worker, tmp_path, identifier)["outcome"] == "PASS"
