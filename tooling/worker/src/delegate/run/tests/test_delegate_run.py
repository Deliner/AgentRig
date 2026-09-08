import hashlib
import json
import os
from pathlib import Path

import pytest

from tooling.worker.src.delegate.testing.consumer import (
    CLAUDE_RESPONSE,
    call,
    consumer,
    select_claude,
    terminal,
)


@pytest.mark.parametrize("mode", ["read", "artifacts"])
def test_claude_structured_result_and_artifacts_survive_cleanup(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, mode: str
) -> None:
    script = 'test "$1" = --print\ntest "$CLAUDE_CONFIG_DIR" = /claude\ntest "$CLAUDE_CODE_OAUTH_TOKEN" = unused-fixture-key\n! echo changed > /claude/settings.json\n! echo changed > /claude/mcp.json\nprintf artifact > /work/asset.txt\n'
    consumer(worker, tmp_path, monkeypatch, script + CLAUDE_RESPONSE)
    select_claude(tmp_path, monkeypatch)
    path = tmp_path / "delegate.yaml"
    path.write_text(path.read_text().replace('mode: "artifacts"', f'mode: "{mode}"'))
    request_path = tmp_path / "request.json"
    request = json.loads(request_path.read_text())
    reading = mode == "read"
    if reading:
        request["contract"]["artifacts"] = {}
    request_path.write_text(json.dumps(request))
    identifier = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    result = terminal(worker, tmp_path, identifier)
    assert result["outcome"] == "PASS", result
    directory = tmp_path / ".agentrig/runtime/jobs" / identifier
    assert not (directory / "private").exists()
    assert call(worker, tmp_path, "result", identifier)["report"] == result["report"]
    not_reading = not reading
    if not_reading:
        assert (directory / "artifacts/asset.txt").read_text() == "artifact"


@pytest.mark.parametrize(
    "payload",
    [
        "not-json",
        '{"is_error":true}',
        '{"is_error":false}',
        '{"is_error":false,"structured_output":{"ok":false}}',
    ],
)
def test_claude_client_success_does_not_bypass_result_validation(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, payload: str
) -> None:
    consumer(
        worker,
        tmp_path,
        monkeypatch,
        "printf artifact > /work/asset.txt\nprintf '%s' '" + payload + "'\n",
    )
    select_claude(tmp_path, monkeypatch)
    identifier = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    result = terminal(worker, tmp_path, identifier)
    assert result["outcome"] == "ERROR", result
    assert not (tmp_path / ".agentrig/runtime/jobs" / identifier / "private").exists()


@pytest.mark.parametrize("cancel", [False, True])
def test_claude_timeout_and_cancel_use_existing_recovery(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, cancel: bool
) -> None:
    consumer(worker, tmp_path, monkeypatch, "/tools/sleep 60\n")
    select_claude(tmp_path, monkeypatch)
    identifier = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    if cancel:
        call(worker, tmp_path, "cancel", identifier)
    result = terminal(worker, tmp_path, identifier)
    expected = "CANCELLED" if cancel else "ERROR"
    assert result["outcome"] == expected, result
    assert result["report"]["cleanup_errors"] == []


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
