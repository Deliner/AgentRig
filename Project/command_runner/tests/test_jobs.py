import json
import os
import signal
import subprocess
import time
from pathlib import Path
from typing import Any

import pytest

from Project.command_runner.tests.consumer import (
    background_id,
    wait_for_background_output,
)
from Project.shared.testing.consumer import CONFIG, invoke, project


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


def run_leaky(worker: Path, root: Path, code: str) -> subprocess.Popen[bytes]:
    argv = json.dumps(["python3", "-c", code])
    project(root, CONFIG + f"\n  leaky:\n    argv: {argv}\n")
    return subprocess.Popen(
        [str(worker), "run", "--root", str(root), "leaky"],
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def terminate(process: subprocess.Popen[bytes]) -> None:
    running = process.poll() is None
    if running:
        process.kill()
        process.communicate()


def wait_until_draining(process: subprocess.Popen[bytes]) -> bool:
    assert process.stdout is not None
    deadline = time.monotonic() + 10
    seen_exit = False
    ticks_after = 0
    while time.monotonic() < deadline and ticks_after < 3:
        line = process.stdout.readline()
        empty = not line
        if empty:
            break
        text = line.decode(errors="replace").strip()
        exited = text == "main-done"
        if exited:
            seen_exit = True
        tick = seen_exit and text == "tick"
        if tick:
            ticks_after += 1
    ready = seen_exit and ticks_after >= 3
    return ready


# DECISION: D095
def test_foreground_drain_is_bounded_when_descendant_holds_pipe(
    worker: Path, tmp_path: Path
) -> None:
    leak = (
        "import subprocess,sys\n"
        "subprocess.Popen(['sleep', '30'])\n"
        "sys.stdout.write('main-done\\n')\n"
        "sys.stdout.flush()\n"
        "sys.exit(0)\n"
    )
    process = run_leaky(worker, tmp_path, leak)
    try:
        process.communicate(timeout=10)
        assert process.returncode == 0
    finally:
        terminate(process)


# DECISION: D095
def test_worker_stays_signal_controllable_after_child_exits(worker: Path, tmp_path: Path) -> None:
    chatty = (
        "import subprocess,sys\n"
        "subprocess.Popen(['sh', '-c', 'for i in $(seq 200); do echo tick; sleep 0.05; done'])\n"
        "sys.stdout.write('main-done\\n')\n"
        "sys.stdout.flush()\n"
        "sys.exit(0)\n"
    )
    process = run_leaky(worker, tmp_path, chatty)
    try:
        draining = wait_until_draining(process)
        assert draining, "worker did not reach the post-exit drain"
        process.send_signal(signal.SIGTERM)
        process.wait(timeout=5)
        assert process.returncode == -signal.SIGTERM
    finally:
        terminate(process)


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


def test_owner_cleanup_stops_nested_background_run(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("WORKER_OWNER", "nested-group-owner")
    nested = [str(worker), "job-start", "--root", str(tmp_path), "wait"]
    config = CONFIG + '  wait:\n    argv: ["sleep", "60"]\n  nested:\n    argv: '
    project(tmp_path, config + json.dumps(nested))
    result = invoke(worker, tmp_path, "run", "nested")
    assert result.returncode == 0, result.stderr
    identifier = json.loads(result.stdout)["run_id"]
    try:
        rows = json.loads(invoke(worker, tmp_path, "jobs").stdout)
        records = {row["command"]: row for row in rows}
        parent, child = records["nested"], records["wait"]
        assert child["parent_run"] == parent["run_id"]
        assert parent["owner"] == child["owner"] == "nested-group-owner"
        assert child["group"]["pid"] == child["supervisor"]["pid"]
        cleanup = invoke(worker, tmp_path, "job-cleanup")
        assert cleanup.returncode == 0, cleanup.stderr
        assert identifier in json.loads(cleanup.stdout)["stopped"]
        status = json.loads(invoke(worker, tmp_path, "job-status", identifier).stdout)
        assert status["state"] == "cancelled"
        assert status["group_observation"]["populated"] is False
    finally:
        assert invoke(worker, tmp_path, "job-stop", identifier).returncode == 0


def test_background_ownership_and_descendant_cleanup(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    code = "import subprocess,time; subprocess.Popen(['sleep','60']); print('ready',flush=True); time.sleep(60)"
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
        assert result["group_observation"]["populated"] is False
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
        assert state["group_observation"]["populated"] is False
        logs = json.loads(invoke(worker, tmp_path, "job-logs", identifier).stdout)
        assert logs["stdout"]["text"] == "ready\n"
    finally:
        result = invoke(worker, tmp_path, "job-stop", identifier)
        assert result.returncode == 0, result.stderr


def test_cleanup_preserves_another_branch(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
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
