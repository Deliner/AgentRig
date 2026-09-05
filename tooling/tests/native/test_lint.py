from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path
from typing import Any

import pytest

# DECISION: D016
ROOT = Path(__file__).parents[3]
SKILL = ".agents/skills/refactor-large-file/SKILL.md"
CONFIG = f"""version = 1
config_skill = ".agents/skills/repair/SKILL.md"

[[rules]]
id = "source"
kind = "nonblank-lines"
target = "file"
include = ["src/**"]
extensions = [".rs", ".py", ".ts"]
warning = 3
error = 5
warning_skill = "{SKILL}"
error_skill = "{SKILL}"
"""


def prepare(root: Path, config: str = CONFIG) -> None:
    shutil.copytree(ROOT / ".agents/skills", root / ".agents/skills")
    (root / "src").mkdir()
    (root / "lint.toml").write_text(config, encoding="utf-8")


def lint(worker: Path, root: Path) -> tuple[int, list[dict[str, Any]]]:
    output = subprocess.run(
        [str(worker), "lint", "--root", str(root), "--config", "lint.toml", "--json"],
        text=True,
        capture_output=True,
        check=False,
    )
    return output.returncode, json.loads(output.stdout)


@pytest.mark.parametrize(
    ("lines", "level", "code"),
    [(3, None, 0), (4, "warning", 0), (5, "warning", 0), (6, "error", 1)],
)
# INVARIANT: I004
def test_native_lint_thresholds_and_skills(
    worker: Path, tmp_path: Path, lines: int, level: str | None, code: int
) -> None:
    prepare(tmp_path)
    for extension in ["rs", "py", "ts"]:
        (tmp_path / f"src/example.{extension}").write_text("line\n\n" * lines, encoding="utf-8")
    (tmp_path / "src/ignored.bin").write_bytes(b"\xff" * 100)
    actual_code, diagnostics = lint(worker, tmp_path)
    assert actual_code == code
    assert len(diagnostics) == (0 if level is None else 3)
    for item in diagnostics:
        assert item["level"] == level
        assert item["skill"] == SKILL
        assert item["actual"] == lines
        assert item["rule"] == "source"


def test_overrides_and_exclusions(worker: Path, tmp_path: Path) -> None:
    config = CONFIG.replace(
        'extensions = [".rs", ".py", ".ts"]',
        'extensions = [".rs", ".py", ".ts"]\nexclude = ["src/excluded.rs"]',
    )
    config += """
[[rules.overrides]]
include = ["src/**"]
extensions = [".rs"]
warning = 7
error = 9
[[rules.overrides]]
include = ["src/special.rs"]
warning = 10
error = 12
"""
    prepare(tmp_path, config)
    for name in ["ordinary.rs", "ordinary.py", "special.rs", "excluded.rs"]:
        (tmp_path / "src" / name).write_text("line\n" * 8, encoding="utf-8")
    code, items = lint(worker, tmp_path)
    assert code == 1
    assert {item["path"]: item["level"] for item in items} == {
        "src/ordinary.rs": "warning",
        "src/ordinary.py": "error",
    }


@pytest.mark.parametrize(
    ("old", "new"),
    [
        ('kind = "nonblank-lines"', 'kind = "unknown"'),
        ('target = "file"', 'target = "directory"'),
        ("warning = 3", "warning = 5"),
        ("warning = 3", "warning = -1"),
        ("warning = 3", "warn = 3"),
        ('include = ["src/**"]', 'include = ["["]'),
        ('include = ["src/**"]', "include = []"),
        ("version = 1", "version = 99"),
        (f'error_skill = "{SKILL}"', 'error_skill = "missing/SKILL.md"'),
        (f'warning_skill = "{SKILL}"', ""),
    ],
)
# INVARIANT: I014
def test_invalid_config_is_actionable(worker: Path, tmp_path: Path, old: str, new: str) -> None:
    prepare(tmp_path, CONFIG.replace(old, new))
    code, items = lint(worker, tmp_path)
    assert code == 2
    assert items[0]["rule"] == "configuration"
    assert items[0]["skill"].endswith("repair/SKILL.md")


def test_directory_counts_immediate_children(worker: Path, tmp_path: Path) -> None:
    config = CONFIG.replace("nonblank-lines", "directory-entries").replace(
        'target = "file"', 'target = "directory"'
    )
    config = config.replace('include = ["src/**"]', 'include = ["src"]').replace(
        'extensions = [".rs", ".py", ".ts"]', ""
    )
    prepare(tmp_path, config)
    (tmp_path / "src/group").mkdir()
    for index in range(20):
        (tmp_path / f"src/group/{index}.txt").touch()
    assert lint(worker, tmp_path) == (0, [])
    for index in range(5):
        (tmp_path / f"src/{index}.txt").touch()
    code, items = lint(worker, tmp_path)
    assert code == 1
    assert items[0]["actual"] == 6


def test_staged_files_and_config_are_isolated(worker: Path, tmp_path: Path) -> None:
    root = tmp_path / "repository"
    root.mkdir()
    prepare(root)
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    source = root / "src/example.rs"
    source.write_text("line\n" * 6, encoding="utf-8")
    subprocess.run(["git", "add", "."], cwd=root, check=True)
    # Make the worktree pass; the staged snapshot must still fail.
    source.write_text("line\n", encoding="utf-8")
    (root / "lint.toml").write_text(CONFIG.replace("error = 5", "error = 50"), encoding="utf-8")
    assert lint(worker, root)[0] == 0
    snapshot = tmp_path / "staged"
    snapshot.mkdir()
    subprocess.run(
        ["git", "checkout-index", "--all", f"--prefix={snapshot}/"], cwd=root, check=True
    )
    code, items = lint(worker, snapshot)
    assert code == 1
    assert items[0]["actual"] == 6
    assert items[0]["limit"] == 5


def test_effective_override_thresholds_must_be_valid(worker: Path, tmp_path: Path) -> None:
    prepare(tmp_path, CONFIG + '\n[[rules.overrides]]\ninclude = ["src/**"]\nwarning = 9\n')
    (tmp_path / "src/example.rs").write_text("line", encoding="utf-8")
    assert lint(worker, tmp_path)[0] == 2


def test_directory_rule_rejects_extension_filter(worker: Path, tmp_path: Path) -> None:
    prepare(
        tmp_path,
        CONFIG.replace("nonblank-lines", "directory-entries").replace(
            'target = "file"', 'target = "directory"'
        ),
    )
    assert lint(worker, tmp_path)[0] == 2
