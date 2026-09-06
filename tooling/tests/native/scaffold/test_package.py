# DECISION: D023
import hashlib
import json
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
        ("agentrig.yaml", "python3", "missing-tool-xyz", "MISSING"),
        ("agentrig.yaml", "runtime: 0.3.0", "runtime: 999.0.0", "project pins"),
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
    binary = tmp_path / ".worker/bin/agentrig"
    binary.write_text("#!/bin/sh\necho agentrig 999.0.0\n")
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
        args = ["--skills", ".worker/bin/agentrig"]
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "init", *args)
    assert result.returncode == 2
    assert file_contents(tmp_path) == before


# INVARIANT: I020
def test_installation_manifest_records_ownership(worker: Path, tmp_path: Path) -> None:
    result = invoke(worker, tmp_path, "init", "--skills", "guides", "--memory", "notes")
    assert result.returncode == 0, result.stderr
    manifest = json.loads((tmp_path / ".worker/manifest.json").read_text())
    assert manifest["manifest_version"] == 1
    assert manifest["package_version"] == "0.3.0"
    assert manifest["config_schema"] == 1
    entries = manifest["files"]
    assert ".worker/manifest.json" not in entries
    assert entries[".worker/bin/agentrig"]["ownership"] == "runtime"
    assert entries[".worker/.gitignore"]["ownership"] == "asset"
    assert entries["agentrig.yaml"]["ownership"] == "configuration"
    assert entries[".worker/lint.yaml"]["ownership"] == "configuration"
    assert entries["guides/repair/SKILL.md"]["ownership"] == "editable"
    assert entries[".worker/hooks/pre-commit"]["ownership"] == "editable"
    assert entries["notes/State.md"]["ownership"] == "memory"
    for relative, entry in entries.items():
        path = tmp_path / relative
        assert hashlib.sha256(path.read_bytes()).hexdigest() == entry["sha256"]
        assert bool(path.stat().st_mode & 0o111) == entry["executable"]
    skill = tmp_path / "guides/repair/SKILL.md"
    skill.write_text(skill.read_text() + "\nLocal instruction.\n")
    assert (
        hashlib.sha256(skill.read_bytes()).hexdigest()
        != entries["guides/repair/SKILL.md"]["sha256"]
    )


def test_existing_manifest_is_an_init_collision(worker: Path, tmp_path: Path) -> None:
    path = tmp_path / ".worker/manifest.json"
    path.parent.mkdir()
    path.write_text("existing receipt")
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "init")
    assert result.returncode == 2
    assert "collision" in result.stderr
    assert file_contents(tmp_path) == before
