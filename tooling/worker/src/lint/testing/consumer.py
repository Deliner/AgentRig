from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path
from typing import Any

ROOT = Path(__file__).parents[5]
SKILL = ".agents/skills/refactor-large-file/SKILL.md"
CONFIG = f"""version: 1
config_skill: ".agents/skills/repair/SKILL.md"

rules:
  - id: "source"
    kind: "nonblank-lines"
    target: "file"
    include: ["src/**"]
    extensions: [".rs", ".py", ".ts"]
    warning: 3
    error: 5
    warning_skill: "{SKILL}"
    error_skill: "{SKILL}"
"""


def prepare(root: Path, config: str = CONFIG) -> None:
    shutil.copytree(ROOT / ".agents/skills", root / ".agents/skills")
    (root / "src").mkdir()
    (root / "lint.yaml").write_text(config, encoding="utf-8")


def lint(worker: Path, root: Path) -> tuple[int, list[dict[str, Any]]]:
    output = subprocess.run(
        [str(worker), "lint", "--root", str(root), "--config", "lint.yaml", "--json"],
        text=True,
        capture_output=True,
        check=False,
    )
    return output.returncode, json.loads(output.stdout)


def explain(worker: Path, root: Path, path: str) -> dict[str, Any]:
    result = subprocess.run(
        [str(worker), "lint-explain", path, "--root", str(root), "--config", "lint.yaml", "--json"],
        capture_output=True,
        text=True,
        check=True,
    )
    value: dict[str, Any] = json.loads(result.stdout)
    return value


def configure(root: Path, kind: str, thresholds: str = 'level: "error"') -> None:
    config = (
        CONFIG.replace("nonblank-lines", kind)
        .replace('extensions: [".rs", ".py", ".ts"]', 'extensions: [".rs", ".py", ".pyi"]')
        .replace("warning: 3\n    error: 5", thresholds.replace("\n", "\n    "))
    )
    prepare(root, config)
