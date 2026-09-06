import json
import os
import shlex
import signal
import subprocess
import time
from pathlib import Path
from typing import Any

import pytest
from support import CONFIG, file_contents, git, invoke, project
from test_gate import GATE
from test_memory import committed_memory, memory


def repository(root: Path, vcs: str = "git") -> None:
    project(root, CONFIG + GATE.replace("warning: true", "warning: false"))
    memory(root)
    (root / ".gitignore").write_text(".runtime/\n")
    (root / "src/value.py").write_text("value = 1\n")
    git_commands = [
        ("init", "-qb", "trunk"),
        ("config", "user.name", "Test"),
        ("config", "user.email", "test@example.invalid"),
        ("add", "."),
        ("commit", "-qm", "baseline"),
    ]
    using_git = vcs == "git"
    commands = (
        git_commands
        if using_git
        else [("init",), ("add", "."), ("commit", "-m", "baseline", "-u", "Test")]
    )
    needs_mercurial_ignore = not using_git
    if needs_mercurial_ignore:
        (root / ".hgignore").write_text("syntax: glob\n.runtime/**\nsrc/ignored.py\n")
    for args in commands:
        subprocess.run([vcs, *args], cwd=root, capture_output=True, check=True)


def resumed(worker: Path, root: Path) -> dict[str, Any]:
    result = invoke(worker, root, "resume")
    assert result.returncode == 0, result.stderr
    value: dict[str, Any] = json.loads(result.stdout)
    return value


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


def revision(root: Path, vcs: str) -> str:
    using_git = vcs == "git"
    args = ["rev-parse", "HEAD"] if using_git else ["log", "-r", ".", "-T", "{node}"]
    return subprocess.check_output([vcs, *args], cwd=root, text=True).strip()


def commit(root: Path, vcs: str) -> str:
    using_git = vcs == "git"
    commands = (
        [("add", "-u"), ("commit", "-qm", "candidate")]
        if using_git
        else [("commit", "-m", "candidate", "-u", "Test")]
    )
    for args in commands:
        subprocess.run([vcs, *args], cwd=root, capture_output=True, check=True)
    return revision(root, vcs)


def evidence(root: Path) -> dict[str, Any]:
    value: dict[str, Any] = json.loads((root / ".runtime/checks.json").read_text())
    return value


@pytest.mark.parametrize("vcs", ["git", "hg"])
def test_revision_gate_uses_exact_files_config_and_repeatable_diagnostics(
    worker: Path, tmp_path: Path, vcs: str
) -> None:
    repository(tmp_path, vcs)
    base = revision(tmp_path, vcs)
    source = tmp_path / "src/value.py"
    source.write_text("line\n" * 61)
    candidate = commit(tmp_path, vcs)
    source.write_text("value = 1\n")
    lint = tmp_path / "lint.yaml"
    lint.write_text(lint.read_text().replace("error: 60", "error: 100"))
    result = invoke(worker, tmp_path, "check", "--revision", candidate, "--only", "lint")
    assert result.returncode == 1, result.stdout + result.stderr
    assert "exceeds 60" in result.stdout
    rerun = result.stderr.split("RERUN: ", 1)[1].splitlines()[0]
    assert f"--revision {candidate}" in rerun
    repeated = subprocess.run(shlex.split(rerun), capture_output=True, text=True, check=False)
    assert repeated.returncode == 1, repeated.stderr
    record = evidence(tmp_path)
    assert record["revision"] == candidate and record["revision_export"]
    assert record["status"] == "completed" and record["code"] == 1
    assert not record["staged"] and record["index_fingerprint"] is None
    assert source.read_text() == "value = 1\n"
    assert "error: 100" in lint.read_text()
    assert invoke(worker, tmp_path, "check", "--revision", base, "--only", "lint").returncode == 0
    checks = resumed(worker, tmp_path)["checks"]
    assert checks["last_run"]["revision"] == base
    assert not checks["revision_matches"] and not checks["full_gate_passed"]


@pytest.mark.parametrize("vcs", ["git", "hg"])
def test_revision_gate_evidence_distinguishes_failures_success_and_mutated_exports(
    worker: Path, tmp_path: Path, vcs: str
) -> None:
    repository(tmp_path, vcs)
    base = revision(tmp_path, vcs)
    assert invoke(worker, tmp_path, "check", "--revision", base).returncode == 23
    assert evidence(tmp_path)["code"] == 23
    config = tmp_path / "agentrig.yaml"
    config.write_text(config.read_text().replace("exit 23", "exit 0"))
    passing = commit(tmp_path, vcs)
    result = invoke(worker, tmp_path, "check", "--revision", passing)
    assert result.returncode == 0, result.stderr
    assert resumed(worker, tmp_path)["checks"]["full_gate_passed"]
    config.write_text(config.read_text().replace("exit 0", "echo changed > src/value.py"))
    changing = commit(tmp_path, vcs)
    result = invoke(worker, tmp_path, "check", "--revision", changing)
    assert result.returncode == 2, result.stdout + result.stderr
    assert "checked revision inputs changed" in result.stderr
    assert evidence(tmp_path)["status"] == "inputs-changed"
    assert not resumed(worker, tmp_path)["checks"]["full_gate_passed"]
    assert (tmp_path / "src/value.py").read_text() == "value = 1\n"


@pytest.mark.parametrize("vcs", ["git", "hg"])
@pytest.mark.parametrize("damage", ["identity", "detail", "removal"])
def test_revision_memory_checks_parent_history_instead_of_candidate_or_checkout(
    worker: Path, tmp_path: Path, vcs: str, damage: str
) -> None:
    notes = committed_memory(worker, tmp_path, vcs)
    base = revision(tmp_path, vcs)
    result = invoke(worker, tmp_path, "check", "--revision", base)
    assert result.returncode == 0, result.stderr
    index = notes / "Decisions.md"
    detail = notes / "Decisions/001.md"
    changes = {
        "identity": (index, index.read_text().replace("| Choice |", "| Changed |")),
        "detail": (detail, detail.read_text().replace("Text.", "Changed.")),
        "removal": (index, "# Decisions\n\n| ID | Decision | Applies in |\n"),
    }
    target, content = changes[damage]
    original = target.read_text()
    target.write_text(content)
    candidate = commit(tmp_path, vcs)
    target.write_text(original)
    result = invoke(worker, tmp_path, "check", "--revision", candidate)
    assert result.returncode == 2, result.stdout + result.stderr
    assert "committed decision" in result.stderr
    assert target.read_text() == original
    result = invoke(worker, tmp_path, "check", "--revision", base)
    assert result.returncode == 0, result.stderr


@pytest.mark.parametrize(
    "args",
    [
        ("--revision",),
        ("--revision", "missing"),
        ("--revision", "HEAD", "--revision", "HEAD"),
        ("--staged", "--revision", "HEAD"),
    ],
)
def test_revision_selection_rejects_invalid_or_conflicting_inputs(
    worker: Path, tmp_path: Path, args: tuple[str, ...]
) -> None:
    repository(tmp_path)
    result = invoke(worker, tmp_path, "check", *args)
    assert result.returncode == 2
    assert "PASS [" not in result.stdout
    assert not (tmp_path / ".runtime/checks.json").exists()


@pytest.mark.parametrize("vcs", ["git", "hg"])
def test_revision_memory_checks_the_second_merge_parent(
    worker: Path, tmp_path: Path, vcs: str
) -> None:
    notes = committed_memory(worker, tmp_path, vcs)
    base = revision(tmp_path, vcs)
    source = tmp_path / "src/lib.rs"
    source.write_text(source.read_text() + "\n// left branch\n")
    left = commit(tmp_path, vcs)
    using_git = vcs == "git"
    switch = ["checkout", "-q", base] if using_git else ["update", "--clean", "--rev", base]
    subprocess.run([vcs, *switch], cwd=tmp_path, capture_output=True, check=True)
    index = notes / "Decisions.md"
    index.write_text(index.read_text().replace("| Choice |", "| Changed |"))
    commit(tmp_path, vcs)
    merge = ["merge", "--no-commit", left] if using_git else ["merge", "--rev", left]
    subprocess.run([vcs, *merge], cwd=tmp_path, capture_output=True, check=True)
    candidate = commit(tmp_path, vcs)
    result = invoke(worker, tmp_path, "check", "--revision", candidate)
    assert result.returncode == 2, result.stdout + result.stderr
    assert "committed decision identity cannot change" in result.stderr
