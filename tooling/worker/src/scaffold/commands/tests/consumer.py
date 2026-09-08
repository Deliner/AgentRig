import json
import subprocess
import time
from pathlib import Path

import pytest

from tooling.worker.src.scaffold.testing.consumer import invoke


def background_id(worker: Path, root: Path, command: str = "wait") -> str:
    result = invoke(worker, root, "job-start", command)
    assert result.returncode == 0, result.stderr
    identifier: str = json.loads(result.stdout)["run_id"]
    return identifier


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
