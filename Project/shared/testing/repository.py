import json
import subprocess
from pathlib import Path
from typing import Any

from Project.shared.testing.consumer import (
    CONFIG,
    invoke,
    memory,
    project,
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
        subprocess.run(["git", *args], cwd=root, capture_output=True, check=True)


def resumed(worker: Path, root: Path) -> dict[str, Any]:
    result = invoke(worker, root, "resume")
    assert result.returncode == 0, result.stderr
    value: dict[str, Any] = json.loads(result.stdout)
    return value


def revision(root: Path) -> str:
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()


def commit(root: Path) -> str:
    for args in [("add", "-u"), ("commit", "-qm", "candidate")]:
        subprocess.run(["git", *args], cwd=root, capture_output=True, check=True)
    return revision(root)


def evidence(root: Path) -> dict[str, Any]:
    value: dict[str, Any] = json.loads((root / ".runtime/checks.json").read_text())
    return value


def feature_repository(root: Path) -> str:
    repository(root)
    return revision(root)
