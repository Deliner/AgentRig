import json
import os
import signal
import subprocess
import time
from pathlib import Path

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
    records = [
        json.loads(line) for line in (tmp_path / ".runtime/commands.jsonl").read_text().splitlines()
    ]
    assert [record["exit_code"] for record in records] == [0, 23]
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
