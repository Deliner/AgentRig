from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

# DECISION: D005

RECIPE = re.compile(r"^([a-z][a-z0-9-]*)(?:\s+[^:]*)?:$")
RECIPE_COMMENT = re.compile(r"^# What: \S(?:.*\S)?; Why: \S(?:.*\S)?\.$")
MODES = {"command", "list", "read", "report", "review", "write"}


def catalog(root: Path) -> dict[str, dict[str, Any]]:
    value = json.loads((root / "tooling" / "command_catalog.json").read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError("command catalog must be an object")
    return value


def just_recipes(root: Path) -> dict[str, tuple[str, str]]:
    lines = (root / "justfile").read_text(encoding="utf-8").splitlines()
    recipes: dict[str, tuple[str, str]] = {}
    for index, line in enumerate(lines):
        match = RECIPE.fullmatch(line)
        if match is not None:
            body = lines[index + 1] if index + 1 < len(lines) else ""
            comment = lines[index - 1] if index > 0 else ""
            recipes[match.group(1)] = body, comment
    return recipes


def validate(root: Path) -> list[str]:
    errors: list[str] = []
    entries = catalog(root)
    recipes = just_recipes(root)
    if set(entries) != set(recipes):
        errors.append(
            f"justfile/catalog recipes differ: just={sorted(recipes)} catalog={sorted(entries)}"
        )
    for name, entry in entries.items():
        mode = entry.get("mode")
        if mode not in MODES:
            errors.append(f"{name}: invalid mode {mode!r}")
        if name in recipes:
            body, comment = recipes[name]
            if f"run {name}" not in body:
                errors.append(f"{name}: recipe does not route through command_runner.py")
            if RECIPE_COMMENT.fullmatch(comment) is None:
                errors.append(
                    f"{name}: recipe requires an immediately preceding one-line "
                    "'# What: ...; Why: ....' comment"
                )
        requires = entry.get("requires")
        if not isinstance(requires, list) or not all(isinstance(path, str) for path in requires):
            errors.append(f"{name}: requires must be a list of paths")
            continue
        for relative in requires:
            if not (root / relative).exists():
                errors.append(f"{name}: missing dependency {relative}")
        if mode == "command":
            argv = entry.get("argv")
            if (
                not isinstance(argv, list)
                or not argv
                or not all(isinstance(arg, str) for arg in argv)
            ):
                errors.append(f"{name}: command mode requires non-empty string argv")
    result = subprocess.run(
        ["just", "--justfile", str(root / "justfile"), "--summary"],
        cwd=root,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        errors.append(f"justfile does not parse: {result.stderr.strip()}")
    return errors


def main() -> int:
    root = Path(sys.argv[1]).resolve() if len(sys.argv) == 2 else Path.cwd()
    try:
        errors = validate(root)
    except (OSError, ValueError, json.JSONDecodeError) as exception:
        errors = [str(exception)]
    for message in errors:
        print(f"COMMAND POLICY: {message}", file=sys.stderr)
    return int(bool(errors))


if __name__ == "__main__":
    raise SystemExit(main())
