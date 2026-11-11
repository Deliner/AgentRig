from __future__ import annotations

import shutil
from pathlib import Path

ROOT = Path(__file__).parents[3]
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
