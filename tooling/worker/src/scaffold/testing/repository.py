import json
import subprocess
from pathlib import Path
from typing import Any

from tooling.worker.src.scaffold.testing.consumer import (
    CONFIG,
    invoke,
    memory,
    project,
    update_config,
    vcs_backend,
    vcs_executable,
)

GATE = """
checks:
- id: "lint"
  kind: "lint"
  skill: "guides/repair/SKILL.md"
- id: "tests"
  kind: "command"
  command: "fail"
  skill: "guides/repair/SKILL.md"
  warning: true
"""


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
        update_config(root / "agentrig.yaml", git={"backend": vcs_backend(vcs)})
        (root / ".hgignore").write_text("syntax: glob\n.runtime/**\nsrc/ignored.py\n")
    for args in commands:
        subprocess.run([vcs_executable(vcs), *args], cwd=root, capture_output=True, check=True)


def resumed(worker: Path, root: Path) -> dict[str, Any]:
    result = invoke(worker, root, "resume")
    assert result.returncode == 0, result.stderr
    value: dict[str, Any] = json.loads(result.stdout)
    return value


def revision(root: Path, vcs: str) -> str:
    using_git = vcs == "git"
    args = ["rev-parse", "HEAD"] if using_git else ["log", "-r", ".", "-T", "{node}"]
    return subprocess.check_output([vcs_executable(vcs), *args], cwd=root, text=True).strip()


def commit(root: Path, vcs: str) -> str:
    using_git = vcs == "git"
    commands = (
        [("add", "-u"), ("commit", "-qm", "candidate")]
        if using_git
        else [("commit", "-m", "candidate", "-u", "Test")]
    )
    for args in commands:
        subprocess.run([vcs_executable(vcs), *args], cwd=root, capture_output=True, check=True)
    return revision(root, vcs)


def evidence(root: Path) -> dict[str, Any]:
    value: dict[str, Any] = json.loads((root / ".runtime/checks.json").read_text())
    return value


def hg(root: Path, *args: str) -> str:
    return subprocess.check_output(["hg", *args], cwd=root, text=True).strip()


def initialize_mercurial(worker: Path, root: Path) -> None:
    result = invoke(
        worker,
        root,
        "init",
        "--vcs",
        "mercurial",
        "--base",
        "trunk",
        "--prefix",
        "task/",
        "--service",
        "rig space",
    )
    assert result.returncode == 0, result.stderr


def feature_repository(root: Path, vcs: str) -> str:
    repository(root, vcs)
    using_git = vcs == "git"
    update_config(
        root / "agentrig.yaml",
        git={
            "backend": "git" if using_git else "mercurial",
            "base": "trunk" if using_git else "default",
        },
    )
    return commit(root, vcs)
