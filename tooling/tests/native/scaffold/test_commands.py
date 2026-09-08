import json
import os
import signal
import subprocess
import time
from pathlib import Path
from typing import Any

import pytest

from tooling.tests.native.scaffold.support import CONFIG, invoke, project


def test_config_and_command_streams(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    assert invoke(worker, tmp_path, "config-check").returncode == 0
    result = invoke(worker, tmp_path, "run", "echo", "--", "a b", "$(no-shell)", input="input data")
    assert result.returncode == 0, result.stderr
    assert "['a b', '$(no-shell)']" in result.stdout
    assert "input data" in result.stdout
    assert result.stderr == "stderr\n"
    assert invoke(worker, tmp_path, "run", "fail").returncode == 23
    records = json.loads(invoke(worker, tmp_path, "jobs").stdout)
    assert sorted(record["exit_code"] for record in records) == [0, 23]
    assert not (tmp_path / ".runtime/commands.jsonl").exists()
    assert "fail 1 1" in invoke(worker, tmp_path, "report").stdout
    assert invoke(worker, tmp_path, "run", "unknown").returncode == 2
    assert invoke(worker, tmp_path, "run", "fail", "extra").returncode == 2


@pytest.mark.parametrize(
    ("old", "new"),
    [
        ('runtime: "0.3.0"', 'runtime: "9.0.0"'),
        ("version: 1", "version: 7"),
        ("paths:", '\nprocesses:\n  foreground: "unknown"\npaths:\n\n'),
        ('memory: "notes"', 'memory: "../outside"'),
        ("read_only: true", 'read_only: "true"'),
        ("accepts_args: true", "accept_arg: true"),
        ('skills: "guides"', 'skills: "/outside"'),
        ('base: "trunk"', 'base: "bad branch"'),
        ('base: "trunk"', 'base: "topic.lock"'),
        ('prefix: "task/"', 'prefix: "trunk"'),
        ('prefix: "task/"', 'prefix: "task//"'),
        ("read_only: true", 'read_only: true\nlifetime: "forever"'),
    ],
)
def test_configuration_errors(worker: Path, tmp_path: Path, old: str, new: str) -> None:
    assert old in CONFIG
    project(tmp_path, CONFIG.replace(old, new))
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "ACTION:" in result.stderr


def test_read_isolation(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    target = tmp_path / "src/protected"
    target.write_text("original")
    result = invoke(
        worker, tmp_path, "run", "read", "--", "sh", "-c", "echo replaced > src/protected"
    )
    assert result.returncode != 0
    assert target.read_text() == "original"
    result = invoke(worker, tmp_path, "run", "read", "--", "cat", "src/protected")
    assert result.returncode == 0, result.stderr
    assert result.stdout == "original"


def test_signal_reaches_child(worker: Path, tmp_path: Path) -> None:
    project(
        tmp_path,
        CONFIG
        + '\n  wait:\n    argv: ["python3", "-c", "import os,time; open(\\"child.pid\\",\\"w\\").write(str(os.getpid())); time.sleep(60)"]\n',
    )
    process = subprocess.Popen(
        [str(worker), "run", "--root", str(tmp_path), "wait"],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    pid_file = tmp_path / "child.pid"
    try:
        deadline = time.monotonic() + 10
        while not pid_file.exists() and process.poll() is None and time.monotonic() < deadline:
            time.sleep(0.01)
        assert pid_file.exists()
        child_pid = int(pid_file.read_text())
        process.send_signal(signal.SIGTERM)
        process.communicate(timeout=5)
        assert process.returncode == 143
        with pytest.raises(ProcessLookupError):
            os.kill(child_pid, 0)
    finally:
        child_running = process.poll() is None
        if child_running:
            process.kill()
            process.communicate()


def wait_for_job(worker: Path, root: Path) -> dict[str, Any]:
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        result = invoke(worker, root, "jobs")
        assert result.returncode == 0, result.stderr
        rows: list[dict[str, Any]] = json.loads(result.stdout)
        for row in rows:
            running = row["state"] == "running"
            if running:
                return row
        time.sleep(0.01)
    raise AssertionError("managed command did not become visible")


def start(worker: Path, root: Path) -> subprocess.Popen[bytes]:
    project(root, CONFIG + '\n  wait:\n    argv: ["sleep", "60"]\n')
    return subprocess.Popen(
        [str(worker), "run", "--root", str(root), "wait"],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        env={**os.environ, "WORKER_OWNER": "test-owner", "WORKER_PARENT_RUN": "parent-run"},
    )


def test_live_command_identity_and_completion(worker: Path, tmp_path: Path) -> None:
    process = start(worker, tmp_path)
    try:
        row = wait_for_job(worker, tmp_path)
        assert row["owner"] == "test-owner" and row["parent_run"] == "parent-run"
        assert row["supervisor"]["pid"] == process.pid
        assert row["child"]["start_ticks"] > 0
        assert row["leader_resources"]["resident_bytes"] > 0
        assert row["branch"] is None
        process.send_signal(signal.SIGTERM)
        assert process.wait(timeout=5) == 143
        result = invoke(worker, tmp_path, "job-status", row["run_id"])
        finished = json.loads(result.stdout)
        assert finished["state"] == "completed" and finished["exit_code"] == 143
        assert finished["leader_resources"] is None
    finally:
        alive = process.poll() is None
        if alive:
            process.terminate()
            process.wait(timeout=5)


def test_recovery_does_not_trust_saved_pid(worker: Path, tmp_path: Path) -> None:
    process = start(worker, tmp_path)
    row = wait_for_job(worker, tmp_path)
    child = row["child"]["pid"]
    try:
        process.kill()
        process.wait(timeout=5)
        orphan = json.loads(invoke(worker, tmp_path, "job-status", row["run_id"]).stdout)
        assert orphan["state"] == "orphaned"
        path = tmp_path / ".runtime/jobs" / row["run_id"] / "record.json"
        saved = json.loads(path.read_text())
        saved["child"]["start_ticks"] += 1
        path.write_text(json.dumps(saved))
        stale = json.loads(invoke(worker, tmp_path, "job-status", row["run_id"]).stdout)
        assert stale["state"] == "interrupted" and stale["leader_resources"] is None
        os.kill(child, 0)
    finally:
        os.kill(child, signal.SIGKILL)


def test_spawn_failure_is_recorded(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, CONFIG + '\n  missing:\n    argv: ["/missing-worker-test-binary"]\n')
    assert invoke(worker, tmp_path, "run", "missing").returncode == 127
    row = json.loads(invoke(worker, tmp_path, "jobs").stdout)[0]
    assert row["state"] == "completed" and row["exit_code"] == 127
    assert "cannot execute" in row["error"]
    assert invoke(worker, tmp_path, "job-status", "../escape").returncode == 2


def test_nested_command_inherits_owner_and_parent(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("WORKER_OWNER", "nested-owner")
    monkeypatch.setenv("WORKER_PARENT_RUN", "external-parent")
    nested = [str(worker), "run", "--root", str(tmp_path), "fail"]
    project(tmp_path, CONFIG + "\n  nested:\n    argv: " + json.dumps(nested))
    result = invoke(worker, tmp_path, "run", "nested")
    assert result.returncode == 23, result.stderr
    rows = json.loads(invoke(worker, tmp_path, "jobs").stdout)
    records = {row["command"]: row for row in rows}
    parent, child = records["nested"], records["fail"]
    assert parent["owner"] == child["owner"] == "nested-owner"
    assert parent["parent_run"] == "external-parent"
    assert child["parent_run"] == parent["run_id"]
    assert parent["state"] == child["state"] == "completed"


def test_logs_preserve_streams_and_bound_display(worker: Path, tmp_path: Path) -> None:
    code = (
        "import sys; sys.stdout.buffer.write(bytes([255])*131072); sys.stderr.write('error stream')"
    )
    project(tmp_path, CONFIG + "\n  output:\n    argv: " + json.dumps(["python3", "-c", code]))
    result = subprocess.run(
        [str(worker), "run", "--root", str(tmp_path), "output"], capture_output=True, check=False
    )
    assert result.returncode == 0, result.stderr
    assert result.stdout == bytes([255]) * 131072
    assert result.stderr == b"error stream"
    row = json.loads(invoke(worker, tmp_path, "jobs").stdout)[0]
    assert row["duration_seconds"] > 0
    logs = json.loads(invoke(worker, tmp_path, "job-logs", row["run_id"]).stdout)
    assert logs["stdout"]["truncated"] and logs["stdout"]["bytes"] == 131072
    assert len(logs["stdout"]["text"]) == 65536
    assert logs["stderr"]["text"] == "error stream"
    directory = tmp_path / ".runtime/jobs" / row["run_id"]
    assert (directory / "stdout.log").read_bytes() == result.stdout
    assert (directory / "stderr.log").read_bytes() == result.stderr


def background_id(worker: Path, root: Path, command: str = "wait") -> str:
    result = invoke(worker, root, "job-start", command)
    assert result.returncode == 0, result.stderr
    identifier: str = json.loads(result.stdout)["run_id"]
    return identifier


def test_foreground_scope_preserves_input_and_streams(worker: Path, tmp_path: Path) -> None:
    require_user_systemd()
    project(tmp_path, CONFIG + '\nprocesses:\n  foreground: "systemd"\n')
    result = invoke(worker, tmp_path, "run", "echo", "argument", input="input bytes")
    assert result.returncode == 0, result.stderr
    assert result.stdout == "['argument']\ninput bytes\n"
    assert result.stderr == "stderr\n"
    row = json.loads(invoke(worker, tmp_path, "jobs").stdout)[0]
    assert row["state"] == "completed"
    assert row["containment"] == "systemd user scope"
    assert row["scope"]["invocation"]
    assert row["background"] is False
    logs = json.loads(invoke(worker, tmp_path, "job-logs", row["run_id"]).stdout)
    assert logs["stdout"]["text"] == result.stdout
    assert logs["stderr"]["text"] == result.stderr


def test_scope_launcher_exit_does_not_claim_payload_success(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    project(tmp_path, CONFIG + '\nprocesses:\n  foreground: "systemd"\n')
    launcher = tmp_path / "systemd-run"
    launcher.write_text("#!/bin/sh\nexit 0\n")
    launcher.chmod(0o755)
    monkeypatch.setenv("PATH", str(tmp_path) + os.pathsep + os.environ["PATH"])
    result = invoke(worker, tmp_path, "run", "echo")
    assert result.returncode == 127
    assert "failed before adoption" in result.stderr
    row = json.loads(invoke(worker, tmp_path, "jobs").stdout)[0]
    assert row["exit_code"] == 127 and row["child"] is None
    assert "failed before adoption" in row["error"]


def test_owner_cleanup_covers_nested_scopes(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    require_user_systemd()
    monkeypatch.setenv("WORKER_OWNER", "nested-scope-owner")
    nested = [str(worker), "job-start", "--root", str(tmp_path), "wait"]
    config = 'processes:\n  foreground: "systemd"\n' + CONFIG
    config += '  wait:\n    argv: ["sleep", "60"]\n  nested:\n    argv: '
    project(tmp_path, config + json.dumps(nested))
    result = invoke(worker, tmp_path, "run", "nested")
    assert result.returncode == 0, result.stderr
    identifier = json.loads(result.stdout)["run_id"]
    try:
        rows = json.loads(invoke(worker, tmp_path, "jobs").stdout)
        records = {row["command"]: row for row in rows}
        parent, child = records["nested"], records["wait"]
        assert child["parent_run"] == parent["run_id"]
        assert parent["owner"] == child["owner"] == "nested-scope-owner"
        assert parent["scope"]["unit"] != child["scope"]["unit"]
        stopped = invoke(worker, tmp_path, "job-stop", parent["run_id"])
        assert stopped.returncode == 0, stopped.stderr
        assert (
            json.loads(invoke(worker, tmp_path, "job-status", identifier).stdout)["state"]
            == "running"
        )
        cleanup = invoke(worker, tmp_path, "job-cleanup")
        assert cleanup.returncode == 0, cleanup.stderr
        assert identifier in json.loads(cleanup.stdout)["stopped"]
        status = json.loads(invoke(worker, tmp_path, "job-status", identifier).stdout)
        assert status["state"] == "cancelled"
        assert status["scope_observation"]["populated"] is False
    finally:
        assert invoke(worker, tmp_path, "job-stop", identifier).returncode == 0


def test_foreground_scope_cancels_detached_descendant(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    require_user_systemd()
    monkeypatch.setenv("WORKER_OWNER", "foreground-owner")
    code = "import subprocess,time; subprocess.Popen(['sleep','60'],start_new_session=True); print('ready',flush=True); time.sleep(60)"
    config = 'processes:\n  foreground: "systemd"\n' + CONFIG + "\n  wait:\n    argv: "
    project(tmp_path, config + json.dumps(["python3", "-c", code]))
    process = subprocess.Popen(
        [str(worker), "run", "--root", str(tmp_path), "wait"],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    try:
        row = wait_for_job(worker, tmp_path)
        wait_for_background_output(worker, tmp_path, row["run_id"])
        stopped = invoke(worker, tmp_path, "job-stop", row["run_id"])
        assert stopped.returncode == 0, stopped.stderr
        assert json.loads(stopped.stdout)["scope_observation"]["populated"] is False
        assert process.wait(timeout=5) != 0
    finally:
        cleanup = invoke(worker, tmp_path, "job-cleanup")
        assert cleanup.returncode == 0, cleanup.stderr
        process.wait(timeout=5)


def require_user_systemd() -> None:
    result = subprocess.run(
        ["systemctl", "--user", "show", "--property=Version"],
        capture_output=True,
        text=True,
        check=False,
    )
    unavailable = result.returncode != 0
    if unavailable:
        pytest.skip("background scope integration requires a systemd user manager")


def wait_for_background_output(worker: Path, root: Path, identifier: str) -> None:
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        result = invoke(worker, root, "job-logs", identifier)
        assert result.returncode == 0, result.stderr
        ready = "ready" in json.loads(result.stdout)["stdout"]["text"]
        if ready:
            return
        time.sleep(0.01)
    raise AssertionError("background child did not produce readiness output")


def test_background_ownership_and_descendant_cleanup(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    require_user_systemd()
    code = "import subprocess,time; subprocess.Popen(['sleep','60'], start_new_session=True); print('ready',flush=True); time.sleep(60)"
    project(tmp_path, CONFIG + "\n  wait:\n    argv: " + json.dumps(["python3", "-c", code]))
    monkeypatch.setenv("WORKER_OWNER", "owner-one")
    first = background_id(worker, tmp_path)
    monkeypatch.setenv("WORKER_OWNER", "owner-two")
    second = background_id(worker, tmp_path)
    try:
        wait_for_background_output(worker, tmp_path, first)
        wait_for_background_output(worker, tmp_path, second)
        denied = invoke(worker, tmp_path, "job-stop", first)
        assert denied.returncode == 2 and "another owner" in denied.stderr
        assert (
            json.loads(invoke(worker, tmp_path, "job-status", first).stdout)["state"] == "running"
        )
        stopped = invoke(worker, tmp_path, "job-stop", second)
        assert stopped.returncode == 0, stopped.stderr
        result = json.loads(stopped.stdout)
        assert result["state"] == "cancelled"
        assert result["scope_observation"]["populated"] is False
        assert (
            json.loads(invoke(worker, tmp_path, "job-status", first).stdout)["state"] == "running"
        )
    finally:
        for owner, identifier in [("owner-one", first), ("owner-two", second)]:
            monkeypatch.setenv("WORKER_OWNER", owner)
            stopped = invoke(worker, tmp_path, "job-stop", identifier)
            assert stopped.returncode == 0, stopped.stderr


def test_private_runner_cannot_restart_existing_job(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    assert invoke(worker, tmp_path, "run", "fail").returncode == 23
    row = json.loads(invoke(worker, tmp_path, "jobs").stdout)[0]
    result = invoke(worker, tmp_path, "_job-run", row["run_id"])
    assert result.returncode == 2 and "already finished" in result.stderr


def test_background_forced_stop_keeps_logs(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    require_user_systemd()
    code = "import signal,time; signal.signal(signal.SIGTERM,signal.SIG_IGN); print('ready',flush=True); time.sleep(60)"
    project(tmp_path, CONFIG + "\n  wait:\n    argv: " + json.dumps(["python3", "-c", code]))
    monkeypatch.setenv("WORKER_OWNER", "forced-stop-owner")
    identifier = background_id(worker, tmp_path)
    try:
        wait_for_background_output(worker, tmp_path, identifier)
        result = invoke(worker, tmp_path, "job-stop", identifier)
        assert result.returncode == 0, result.stderr
        state = json.loads(result.stdout)
        assert state["state"] == "cancelled"
        assert state["scope_observation"]["populated"] is False
        logs = json.loads(invoke(worker, tmp_path, "job-logs", identifier).stdout)
        assert logs["stdout"]["text"] == "ready\n"
    finally:
        result = invoke(worker, tmp_path, "job-stop", identifier)
        assert result.returncode == 0, result.stderr


def test_cleanup_preserves_another_branch(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    require_user_systemd()
    project(tmp_path, CONFIG + '\n  wait:\n    argv: ["sleep", "60"]\n')
    subprocess.run(["git", "init", "-q", "-b", "task/first", str(tmp_path)], check=True)
    monkeypatch.setenv("WORKER_OWNER", "branch-owner")
    first = background_id(worker, tmp_path)
    subprocess.run(
        ["git", "symbolic-ref", "HEAD", "refs/heads/task/second"], cwd=tmp_path, check=True
    )
    second = background_id(worker, tmp_path)
    try:
        result = invoke(worker, tmp_path, "job-cleanup", "--branch", "task/first")
        assert result.returncode == 0, result.stderr
        assert (
            json.loads(invoke(worker, tmp_path, "job-status", first).stdout)["state"] == "cancelled"
        )
        assert (
            json.loads(invoke(worker, tmp_path, "job-status", second).stdout)["state"] == "running"
        )
    finally:
        for identifier in [first, second]:
            assert invoke(worker, tmp_path, "job-stop", identifier).returncode == 0
