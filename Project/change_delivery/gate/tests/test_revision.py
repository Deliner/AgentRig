import shlex
import subprocess
from pathlib import Path

import pytest

from Project.shared.testing.consumer import invoke
from Project.shared.testing.repository import (
    commit,
    evidence,
    repository,
    resumed,
    revision,
)


def test_revision_gate_uses_exact_files_config_and_repeatable_diagnostics(
    worker: Path, tmp_path: Path
) -> None:
    repository(tmp_path)
    base = revision(tmp_path)
    source = tmp_path / "src/value.py"
    source.write_text("line\n" * 61)
    candidate = commit(tmp_path)
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
    assert record["revision"] == candidate and record["source"] == "revision"
    assert record["status"] == "completed" and record["code"] == 1
    assert source.read_text() == "value = 1\n"
    assert "error: 100" in lint.read_text()
    assert invoke(worker, tmp_path, "check", "--revision", base, "--only", "lint").returncode == 0
    checks = resumed(worker, tmp_path)["checks"]
    assert checks["last_run"]["revision"] == base
    assert not checks["revision_matches"] and not checks["full_gate_passed"]


def test_revision_gate_evidence_distinguishes_failures_success_and_mutated_exports(
    worker: Path, tmp_path: Path
) -> None:
    repository(tmp_path)
    base = revision(tmp_path)
    assert invoke(worker, tmp_path, "check", "--revision", base).returncode == 23
    assert evidence(tmp_path)["code"] == 23
    config = tmp_path / ".agentrig/commands.yaml"
    config.write_text(config.read_text().replace("exit 23", "exit 0"))
    passing = commit(tmp_path)
    result = invoke(worker, tmp_path, "check", "--revision", passing)
    assert result.returncode == 0, result.stderr
    assert resumed(worker, tmp_path)["checks"]["full_gate_passed"]
    config.write_text(config.read_text().replace("exit 0", "echo changed > src/value.py"))
    changing = commit(tmp_path)
    result = invoke(worker, tmp_path, "check", "--revision", changing)
    assert result.returncode == 2, result.stdout + result.stderr
    assert "checked revision inputs changed" in result.stderr
    assert evidence(tmp_path)["status"] == "inputs-changed"
    assert not resumed(worker, tmp_path)["checks"]["full_gate_passed"]
    assert (tmp_path / "src/value.py").read_text() == "value = 1\n"


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
