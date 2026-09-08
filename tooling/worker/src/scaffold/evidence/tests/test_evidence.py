import os
import signal
import subprocess
import time
from pathlib import Path

import pytest

from tooling.worker.src.scaffold.testing.consumer import (
    file_contents,
    git,
    invoke,
)
from tooling.worker.src.scaffold.testing.repository import repository, resumed


@pytest.mark.parametrize("vcs", ["git", "hg"])
# INVARIANT: I018
def test_evidence_tracks_content_and_partial_checks(worker: Path, tmp_path: Path, vcs: str) -> None:
    repository(tmp_path, vcs)
    assert resumed(worker, tmp_path)["checks"]["status"] == "absent"
    assert invoke(worker, tmp_path, "check", "--only", "lint").returncode == 0
    checks = resumed(worker, tmp_path)["checks"]
    assert checks["current"]
    assert not checks["full_gate_passed"]
    assert checks["last_run"]["only"] == "lint"
    source = tmp_path / "src/value.py"
    source.write_text("value = 2\n")
    assert not resumed(worker, tmp_path)["checks"]["current"]
    source.write_text("value = 1\n")
    assert resumed(worker, tmp_path)["checks"]["current"]
    source.chmod(0o755)
    assert not resumed(worker, tmp_path)["checks"]["current"]


def test_mercurial_evidence_tracks_revision_and_excludes_metadata(
    worker: Path, tmp_path: Path
) -> None:
    repository(tmp_path, "hg")
    assert invoke(worker, tmp_path, "check", "--only", "lint").returncode == 0
    expected = subprocess.check_output(
        ["hg", "log", "-r", ".", "-T", "{node}"], cwd=tmp_path, text=True
    )
    assert resumed(worker, tmp_path)["checks"]["last_run"]["revision"] == expected
    (tmp_path / ".hg/extra-metadata").write_text("not source")
    (tmp_path / "src/ignored.py").write_text("ignored\n")
    assert resumed(worker, tmp_path)["checks"]["current"]
    source = tmp_path / "src/value.py"
    source.write_text("value = 2\n")
    subprocess.run(
        ["hg", "commit", "-m", "advance", "-u", "Test"],
        cwd=tmp_path,
        capture_output=True,
        check=True,
    )
    source.write_text("value = 1\n")
    checks = resumed(worker, tmp_path)["checks"]
    assert checks["worktree_matches"]
    assert not checks["revision_matches"]
    assert not checks["current"]
    result = invoke(worker, tmp_path, "check", "--staged")
    assert result.returncode == 2
    assert "Mercurial has no staging index" in result.stderr


def test_staged_evidence_does_not_verify_unstaged_source(worker: Path, tmp_path: Path) -> None:
    repository(tmp_path)
    (tmp_path / "src/value.py").write_text("line\n" * 61)
    assert invoke(worker, tmp_path, "check", "--staged", "--only", "lint").returncode == 0
    checks = resumed(worker, tmp_path)["checks"]
    assert checks["last_run"]["staged"]
    assert not checks["worktree_matches"]
    assert checks["revision_matches"]
    git(tmp_path, "restore", "src/value.py")
    before = file_contents(tmp_path)
    assert resumed(worker, tmp_path)["checks"]["current"]
    assert before == file_contents(tmp_path)
    (tmp_path / "src/value.py").write_text("value = 2\n")
    git(tmp_path, "add", "src/value.py")
    (tmp_path / "src/value.py").write_text("value = 1\n")
    checks = resumed(worker, tmp_path)["checks"]
    assert checks["worktree_matches"]
    assert not checks["index_matches"]
    assert not checks["current"]


def test_repeat_report_is_evidence_not_skill_read_claim(worker: Path, tmp_path: Path) -> None:
    repository(tmp_path)
    for _ in range(2):
        assert invoke(worker, tmp_path, "check").returncode == 23
    result = invoke(worker, tmp_path, "report")
    assert "Repeated check failure" in result.stdout
    checks = resumed(worker, tmp_path)["checks"]
    assert checks["last_run"]["results"][-1]["consecutive_failures"] == 2
    assert not checks["full_gate_passed"]
    config = tmp_path / "agentrig.yaml"
    config.write_text(config.read_text().replace("exit 23", "exit 0"))
    assert invoke(worker, tmp_path, "check").returncode == 0
    checks = resumed(worker, tmp_path)["checks"]
    assert checks["full_gate_passed"]
    assert checks["last_run"]["results"][-1]["consecutive_failures"] == 0
    assert "Repeated check failure" not in invoke(worker, tmp_path, "report").stdout
    git(tmp_path, "commit", "--allow-empty", "-qm", "new revision")
    checks = resumed(worker, tmp_path)["checks"]
    assert checks["worktree_matches"]
    assert not checks["revision_matches"]
    assert not checks["full_gate_passed"]


def test_changed_inputs_are_not_a_completed_proof(worker: Path, tmp_path: Path) -> None:
    repository(tmp_path)
    config = tmp_path / "agentrig.yaml"
    config.write_text(config.read_text().replace("exit 23", "echo changed > src/value.py"))
    assert invoke(worker, tmp_path, "check").returncode == 0
    checks = resumed(worker, tmp_path)["checks"]
    assert checks["last_run"]["status"] == "inputs-changed"
    assert not checks["full_gate_passed"]


def await_evidence(root: Path) -> None:
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        ready = (root / "started").exists()
        if ready:
            return
        time.sleep(0.01)
    pytest.fail("check process did not start")


def test_interrupted_attempt_survives_fresh_resume(worker: Path, tmp_path: Path) -> None:
    repository(tmp_path)
    config = tmp_path / "agentrig.yaml"
    config.write_text(config.read_text().replace("exit 23", "echo $$ > started; sleep 30"))
    with subprocess.Popen(
        [str(worker), "check", "--root", str(tmp_path)],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        start_new_session=True,
    ) as process:
        try:
            await_evidence(tmp_path)
        finally:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            child = tmp_path / "started"
            child_started = child.exists()
            if child_started:
                os.killpg(int(child.read_text()), signal.SIGKILL)
    checks = resumed(worker, tmp_path)["checks"]
    assert checks["last_run"]["status"] == "unfinished"
    assert checks["last_run"]["code"] is None
    assert not checks["full_gate_passed"]
