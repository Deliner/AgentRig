import json
import os
import signal
import subprocess
import time
from pathlib import Path
from typing import Any

import pytest
from support import CONFIG, file_contents, git, invoke, project
from test_gate import GATE
from test_memory import memory


def repository(root: Path) -> None:
    project(root, CONFIG + GATE.replace("warning: true", "warning: false"))
    memory(root)
    (root / ".gitignore").write_text(".runtime/\n")
    (root / "src/value.py").write_text("value = 1\n")
    for args in [
        ("init", "-qb", "trunk"),
        ("config", "user.name", "Test"),
        ("config", "user.email", "test@example.invalid"),
        ("add", "."),
        ("commit", "-qm", "baseline"),
    ]:
        git(root, *args)


def resumed(worker: Path, root: Path) -> dict[str, Any]:
    result = invoke(worker, root, "resume")
    assert result.returncode == 0, result.stderr
    value: dict[str, Any] = json.loads(result.stdout)
    return value


# INVARIANT: I018
def test_evidence_tracks_content_and_partial_checks(worker: Path, tmp_path: Path) -> None:
    repository(tmp_path)
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


def test_resume_resolves_state_revision(worker: Path, tmp_path: Path) -> None:
    repository(tmp_path)
    revision = git(tmp_path, "rev-parse", "--short", "HEAD").stdout.strip()
    state = tmp_path / "notes/State.md"
    state.write_text(state.read_text() + f"\nBranch: `trunk`\nRevision: `{revision}`\n")
    value = resumed(worker, tmp_path)
    assert value["snapshot"] == "current"
    assert value["state_revision"]["head_changed"] is False
    git(tmp_path, "commit", "--allow-empty", "-qm", "next revision")
    value = resumed(worker, tmp_path)
    assert value["snapshot"] == "stale"
    assert value["state_revision"]["head_changed"] is True


@pytest.mark.parametrize("operation", ["merge", "rebase"])
# INVARIANT: I019
def test_resume_observes_real_conflict(worker: Path, tmp_path: Path, operation: str) -> None:
    repository(tmp_path)
    source = tmp_path / "src/value.py"
    git(tmp_path, "switch", "-qc", "task/conflict")
    source.write_text("value = 2\n")
    git(tmp_path, "commit", "-am", "feature")
    git(tmp_path, "switch", "trunk")
    source.write_text("value = 3\n")
    git(tmp_path, "commit", "-am", "base")
    git(tmp_path, "switch", "task/conflict")
    assert git(tmp_path, operation, "trunk", success=False).returncode != 0
    before = file_contents(tmp_path)
    value = resumed(worker, tmp_path)
    assert value["git"][f"{operation}_in_progress"]
    assert file_contents(tmp_path) == before
