import subprocess
from pathlib import Path
from typing import Any

import yaml

CONFIG = """version: 1
runtime: "0.3.0"
config_skill: "guides/repair/SKILL.md"
paths:
  sources: ["src/**"]
  memory: "notes"
  skills: "guides"
  lint: "lint.yaml"
  runtime: ".runtime"
git:
  base: "trunk"
  prefix: "task/"
commands:
  echo:
    argv: ["python3", "-c", "import sys; print(repr(sys.argv[1:])); print(sys.stdin.read()); print('stderr', file=sys.stderr)"]
    accepts_args: true
  fail:
    argv: ["sh", "-c", "exit 23"]
  read:
    argv: []
    accepts_args: true
    read_only: true
"""
LINT = """version: 1
config_skill: guides/repair/SKILL.md
exclude:
- .git/**
- .runtime/**
rules:
- id: lines
  kind: nonblank-lines
  target: file
  include:
  - src/**
  warning: 30
  error: 60
  warning_skill: guides/repair/SKILL.md
  error_skill: guides/repair/SKILL.md
"""


def project(root: Path, config: str = CONFIG) -> Path:
    root.mkdir(exist_ok=True)
    skill = root / "guides/repair/SKILL.md"
    skill.parent.mkdir(parents=True)
    skill.write_text("---\nname: repair\ndescription: Fix the reported failing check.\n---\n")
    (root / "src").mkdir()
    (root / "agentrig.yaml").write_text(config)
    (root / "lint.yaml").write_text(LINT)
    return root


def vcs_backend(vcs: str) -> str | dict[str, list[str]]:
    private = vcs == "private"
    if private:
        script = Path(__file__).resolve().parents[3] / "examples/external_vcs.py"
        return {"command": ["python3", "-B", str(script)]}
    return {"git": "git", "hg": "mercurial"}[vcs]


def vcs_executable(vcs: str) -> str:
    return {"private": "hg"}.get(vcs, vcs)


def update_config(path: Path, **changes: Any) -> None:
    source = path.read_text()
    config = yaml.safe_load(source)
    for key, value in changes.items():
        current = config.get(key)
        mapping = isinstance(current, dict) and isinstance(value, dict)
        sequence = isinstance(current, list) and isinstance(value, list)
        if mapping:
            current.update(value)
        elif sequence:
            current.extend(value)
        else:
            config[key] = value
    comments = "\n".join(filter(lambda line: line.startswith("#"), source.splitlines()))
    path.write_text(comments + "\n" + yaml.safe_dump(config, sort_keys=False))


def invoke(
    worker: Path, root: Path, *args: str, input: str = ""
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(worker), args[0], "--root", str(root), *args[1:]],
        input=input,
        text=True,
        capture_output=True,
        check=False,
    )


def git(root: Path, *args: str, success: bool = True) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(["git", *args], cwd=root, capture_output=True, text=True, check=False)
    if success:
        assert result.returncode == 0, result.stdout + result.stderr
    return result


def file_contents(root: Path) -> dict[Path, bytes]:
    contents = {}
    for path in root.rglob("*"):
        file = path.is_file()
        if file:
            contents[path] = path.read_bytes()
    return contents


def memory(root: Path) -> Path:
    path = root / "notes"
    path.mkdir()
    for name, headers in [
        ("Plan", "ID | Status | Depends on | Feature | User capability"),
        ("Decisions", "ID | Decision | Applies in"),
        ("Invariants", "ID | Invariant | Enforced by"),
    ]:
        (path / f"{name}.md").write_text(f"# {name}\n\n| {headers} |\n")
        (path / name).mkdir()
    (path / "State.md").write_text(
        "# State\n\n"
        + "\n\n".join(
            f"## {section}\n\nNone."
            for section in [
                "Focus",
                "Workspace",
                "Progress",
                "Verification",
                "Blockers",
                "Next action",
            ]
        )
    )
    return path
