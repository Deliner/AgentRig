import shlex
import subprocess
from pathlib import Path

import pytest
from support import file_contents, invoke


def test_doctor_observes_registration_and_tools(worker: Path, tmp_path: Path) -> None:
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    assert invoke(worker, tmp_path, "init").returncode == 0
    assert invoke(worker, tmp_path, "doctor").returncode == 0
    changes = [
        (".codex/config.toml", "hooks = true", "hooks = false", "Codex registration"),
        (".codex/hooks.json", "SessionStart", "UnknownEvent", "Codex registration"),
        (".worker/hooks/pre-commit", "--staged", "--incorrect", "Git hooks"),
        ("worker.toml", '"python3"', '"missing-tool-xyz"', "MISSING"),
        ("worker.toml", 'runtime = "0.1.0"', 'runtime = "999.0.0"', "project pins"),
    ]
    for name, before, after, expected in changes:
        path = tmp_path / name
        original = path.read_text()
        assert before in original
        path.write_text(original.replace(before, after))
        verify_doctor_failure(worker, tmp_path, expected)
        path.write_text(original)
    hook = tmp_path / ".worker/hooks/pre-commit"
    hook.chmod(0o644)
    assert invoke(worker, tmp_path, "doctor").returncode == 1
    hook.chmod(0o755)
    binary = tmp_path / ".worker/bin/discipline-worker"
    binary.write_text("#!/bin/sh\necho discipline-worker 999.0.0\n")
    result = invoke(worker, tmp_path, "doctor")
    assert result.returncode == 1
    assert "installed binary: MISSING OR INCOMPATIBLE" in result.stdout


def verify_doctor_failure(worker: Path, root: Path, expected: str) -> None:
    result = invoke(worker, root, "doctor")
    assert result.returncode != 0
    assert expected in result.stdout + result.stderr
    assert ".worker/skills/repair/SKILL.md" in result.stderr
    command = result.stderr.split("RERUN: ", 1)[1].splitlines()[0]
    repeated = subprocess.run(shlex.split(command), capture_output=True, text=True, check=False)
    assert repeated.returncode == result.returncode
    assert expected in repeated.stdout + repeated.stderr


@pytest.mark.parametrize("case", ["invalid-glob", "file-parent", "generated-parent"])
def test_init_rejects_invalid_layout_before_writing(
    worker: Path, tmp_path: Path, case: str
) -> None:
    args = ["--source", "src/["]
    existing_file_parent = case == "file-parent"
    generated_file_parent = case == "generated-parent"
    if existing_file_parent:
        (tmp_path / ".codex").write_text("user data")
        args = []
    elif generated_file_parent:
        args = ["--skills", ".worker/bin/discipline-worker"]
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "init", *args)
    assert result.returncode == 2
    assert file_contents(tmp_path) == before
