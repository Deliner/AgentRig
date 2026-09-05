import json
import os
import signal
import subprocess
import time
from pathlib import Path
from typing import Any

import pytest
from support import CONFIG, invoke, project


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
        ('runtime = "0.2.0"', 'runtime = "9.0.0"'),
        ("version = 1", "version = 7"),
        ('memory = "notes"', 'memory = "../outside"'),
        ("read_only = true", 'read_only = "true"'),
        ("accepts_args = true", "accept_arg = true"),
        ('skills = "guides"', 'skills = "/outside"'),
        ('base = "trunk"', 'base = "bad branch"'),
        ('base = "trunk"', 'base = "topic.lock"'),
        ('prefix = "task/"', 'prefix = "trunk"'),
        ('prefix = "task/"', 'prefix = "task//"'),
    ],
)
def test_configuration_errors(worker: Path, tmp_path: Path, old: str, new: str) -> None:
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
        + "\n[commands.wait]\nargv = ['python3', '-c', 'import os,time; open(\"child.pid\",\"w\").write(str(os.getpid())); time.sleep(60)']\n",
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
    project(root, CONFIG + '\n[commands.wait]\nargv = ["sleep", "60"]\n')
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
    project(tmp_path, CONFIG + '\n[commands.missing]\nargv = ["/missing-worker-test-binary"]\n')
    assert invoke(worker, tmp_path, "run", "missing").returncode == 127
    row = json.loads(invoke(worker, tmp_path, "jobs").stdout)[0]
    assert row["state"] == "completed" and row["exit_code"] == 127
    assert "cannot execute" in row["error"]
    assert invoke(worker, tmp_path, "job-status", "../escape").returncode == 2


def test_logs_preserve_streams_and_bound_display(worker: Path, tmp_path: Path) -> None:
    code = (
        "import sys; sys.stdout.buffer.write(bytes([255])*131072); sys.stderr.write('error stream')"
    )
    project(tmp_path, CONFIG + "\n[commands.output]\nargv = " + json.dumps(["python3", "-c", code]))
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
