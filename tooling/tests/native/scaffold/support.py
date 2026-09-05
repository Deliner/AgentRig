import subprocess
from pathlib import Path

CONFIG = """version = 1
runtime = "0.1.0"
config_skill = "guides/repair/SKILL.md"
[paths]
sources = ["src/**"]
memory = "notes"
skills = "guides"
lint = "lint.toml"
runtime = ".runtime"
[git]
base = "trunk"
prefix = "task/"
[commands.echo]
argv = ["python3", "-c", "import sys; print(repr(sys.argv[1:])); print(sys.stdin.read()); print('stderr', file=sys.stderr)"]
accepts_args = true
[commands.fail]
argv = ["sh", "-c", "exit 23"]
[commands.read]
argv = []
accepts_args = true
read_only = true
"""
LINT = """version = 1
config_skill = "guides/repair/SKILL.md"
exclude = [".git/**", ".runtime/**"]
[[rules]]
id = "lines"
kind = "nonblank-lines"
target = "file"
include = ["src/**"]
warning = 30
error = 60
warning_skill = "guides/repair/SKILL.md"
error_skill = "guides/repair/SKILL.md"
"""


def project(root: Path, config: str = CONFIG) -> Path:
    root.mkdir(exist_ok=True)
    skill = root / "guides/repair/SKILL.md"
    skill.parent.mkdir(parents=True)
    skill.write_text("---\nname: repair\ndescription: Fix the reported failing check.\n---\n")
    (root / "src").mkdir()
    (root / "worker.toml").write_text(config)
    (root / "lint.toml").write_text(LINT)
    return root


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
