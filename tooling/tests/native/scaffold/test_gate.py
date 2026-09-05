import subprocess
from pathlib import Path

from support import CONFIG, invoke, project

GATE = """
[[checks]]
id = "lint"
kind = "lint"
skill = "guides/repair/SKILL.md"
[[checks]]
id = "tests"
kind = "command"
command = "fail"
skill = "guides/repair/SKILL.md"
warning = true
"""


def test_gate_warning_and_block(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, CONFIG + GATE)
    result = invoke(worker, tmp_path, "check")
    assert result.returncode == 0, result.stderr
    assert "WARNING [tests]" in result.stderr
    assert "guides/repair/SKILL.md" in result.stderr
    (tmp_path / "src/large.py").write_text("line\n" * 61)
    result = invoke(worker, tmp_path, "check")
    assert result.returncode == 1
    assert "ERROR [lint]" in result.stderr
    assert "WARNING [tests]" not in result.stderr


def test_staged_gate_uses_staged_configuration(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, CONFIG + GATE)
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    (tmp_path / "src/large.py").write_text("line\n" * 61)
    subprocess.run(["git", "add", "."], cwd=tmp_path, check=True)
    (tmp_path / "src/large.py").write_text("line\n")
    (tmp_path / "lint.toml").write_text(
        (tmp_path / "lint.toml").read_text().replace("error = 60", "error = 100")
    )
    assert invoke(worker, tmp_path, "check").returncode == 0
    result = invoke(worker, tmp_path, "check", "--staged")
    assert result.returncode == 1, result.stderr
    assert "exceeds 60" in result.stdout


def test_missing_stage_command_is_actionable(worker: Path, tmp_path: Path) -> None:
    project(
        tmp_path,
        (CONFIG + GATE).replace(
            'argv = ["sh", "-c", "exit 23"]', 'argv = ["missing-worker-test-executable"]'
        ),
    )
    result = invoke(worker, tmp_path, "check")
    assert result.returncode == 0
    assert "cannot execute" in result.stderr
    assert "guides/repair/SKILL.md" in result.stderr


def test_unknown_stage_command_rejected(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, (CONFIG + GATE).replace('command = "fail"', 'command = "unknown"'))
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "checks.tests: unknown command unknown" in result.stderr
