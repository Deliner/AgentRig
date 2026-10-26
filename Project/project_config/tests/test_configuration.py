from pathlib import Path

import pytest

from Project.shared.testing.consumer import CONFIG, invoke, project


@pytest.mark.parametrize(
    ("old", "new"),
    [
        ('runtime: "0.3.0"', 'runtime: "9.0.0"'),
        ("version: 1", "version: 7"),
        ("paths:", '\nfrontend: "unknown"\npaths:\n'),
        ('memory: "notes"', 'memory: "../outside"'),
        ("read_only: true", 'read_only: "true"'),
        ("accepts_args: true", "accept_arg: true"),
        ('skills: "guides"', 'skills: "/outside"'),
        ('base: "trunk"', 'base: "bad branch"'),
        ('base: "trunk"', 'base: "topic.lock"'),
        ('prefix: "task/"', 'prefix: "trunk"'),
        ('prefix: "task/"', 'prefix: "task//"'),
        ("read_only: true", 'read_only: true\nlifetime: "forever"'),
    ],
)
def test_configuration_errors(worker: Path, tmp_path: Path, old: str, new: str) -> None:
    assert old in CONFIG
    project(tmp_path, CONFIG.replace(old, new))
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "ACTION:" in result.stderr


# DECISION: D038
def test_project_module_rejects_another_module_section(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    path = tmp_path / ".agentrig/project.yaml"
    path.write_text(path.read_text() + "commands: {}\n")
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "move commands to .agentrig/commands.yaml" in result.stderr
