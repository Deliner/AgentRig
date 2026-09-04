from __future__ import annotations

import fcntl
import json
import os
import subprocess
import sys
import time
from collections import defaultdict
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

# DECISION: D005

ROOT = Path(__file__).resolve().parents[1]
LOG_NAME = "agent-command-log.jsonl"
REVIEW_NAME = "agent-command-review.json"
REVIEW_THRESHOLD = 10_000


def load_catalog(root: Path = ROOT) -> dict[str, dict[str, Any]]:
    value = json.loads((root / "tooling" / "command_catalog.json").read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError("command catalog must be an object")
    return value


def git_path(root: Path, name: str) -> Path:
    result = subprocess.run(
        ["git", "rev-parse", "--git-path", name],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    path = Path(result.stdout.strip())
    return path if path.is_absolute() else root / path


def append_log(root: Path, record: dict[str, Any]) -> None:
    path = git_path(root, LOG_NAME)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as output:
        fcntl.flock(output, fcntl.LOCK_EX)
        output.write(json.dumps(record, ensure_ascii=False, separators=(",", ":")) + "\n")
        output.flush()
        os.fsync(output.fileno())


def read_records(root: Path) -> list[dict[str, Any]]:
    path = git_path(root, LOG_NAME)
    if not path.exists():
        return []
    records: list[dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(value, dict):
            records.append(value)
    return records


def sandbox_argv(root: Path, argv: list[str]) -> list[str]:
    command = [
        "bwrap",
        "--ro-bind",
        "/",
        "/",
        "--dev",
        "/dev",
        "--proc",
        "/proc",
        "--unshare-all",
        "--die-with-parent",
    ]
    if not root.is_relative_to("/tmp"):
        command.extend(["--tmpfs", "/tmp"])
    return [*command, "--chdir", str(root), "--", *argv]


def print_catalog(catalog: dict[str, dict[str, Any]]) -> int:
    width = max(map(len, catalog))
    for name, entry in catalog.items():
        print(f"{name:<{width}}  {entry['description']}")
    return 0


def print_report(root: Path) -> int:
    totals: dict[str, list[float]] = defaultdict(list)
    failures: dict[str, int] = defaultdict(int)
    for record in read_records(root):
        recipe = record.get("recipe")
        duration = record.get("duration_seconds")
        if isinstance(recipe, str) and isinstance(duration, int | float):
            totals[recipe].append(float(duration))
            if record.get("exit_code") != 0:
                failures[recipe] += 1
    print("recipe calls failures total_s avg_s max_s")
    for recipe in sorted(totals):
        values = totals[recipe]
        print(
            f"{recipe} {len(values)} {failures[recipe]} "
            f"{sum(values):.3f} {sum(values) / len(values):.3f} {max(values):.3f}"
        )
    changed, base = review_progress(root, initialize=True)
    print(f"command-review diff_lines={changed} threshold={REVIEW_THRESHOLD} base={base}")
    return 0


def head(root: Path) -> str:
    return subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()


def read_review_base(root: Path, initialize: bool) -> str:
    path = git_path(root, REVIEW_NAME)
    if path.exists():
        try:
            value = json.loads(path.read_text(encoding="utf-8"))
            base = value.get("base")
            if isinstance(base, str):
                subprocess.run(
                    ["git", "cat-file", "-e", f"{base}^{{commit}}"],
                    cwd=root,
                    check=True,
                    capture_output=True,
                )
                return base
        except (json.JSONDecodeError, subprocess.CalledProcessError):
            pass
    base = head(root)
    if initialize:
        path.write_text(json.dumps({"base": base}) + "\n", encoding="utf-8")
    return base


def review_progress(root: Path, initialize: bool) -> tuple[int, str]:
    base = read_review_base(root, initialize)
    result = subprocess.run(
        ["git", "diff", "--numstat", base, "--", "."],
        cwd=root,
        check=True,
        capture_output=True,
        text=True,
    )
    changed = 0
    for line in result.stdout.splitlines():
        added, removed, _path = line.split("\t", 2)
        if added.isdigit() and removed.isdigit():
            changed += int(added) + int(removed)
    return changed, base


def reset_review(root: Path) -> None:
    git_path(root, REVIEW_NAME).write_text(
        json.dumps({"base": head(root)}) + "\n", encoding="utf-8"
    )


def remind(root: Path = ROOT) -> int:
    changed, _base = review_progress(root, initialize=True)
    if changed >= REVIEW_THRESHOLD:
        print(
            f"Command catalog review is due: {changed} diff lines since the last review. "
            "Run `just review-commands`.",
            file=sys.stderr,
        )
    return 0


def command_for(root: Path, recipe: str, extra: list[str]) -> tuple[list[str] | None, str]:
    if extra[:1] == ["--"]:
        extra = extra[1:]
    entry = load_catalog(root)[recipe]
    mode = entry["mode"]
    if mode == "list":
        return None, mode
    if mode in {"read", "write"}:
        if not extra:
            raise ValueError(f"{recipe} requires argv after `--`")
        return (sandbox_argv(root, extra) if mode == "read" else extra), mode
    if mode == "command":
        if extra and not entry.get("accepts_args", False):
            raise ValueError(f"{recipe} does not accept arguments")
        return [*entry["argv"], *extra], mode
    if mode in {"report", "review"}:
        return None, mode
    raise ValueError(f"unknown mode for {recipe}: {mode}")


def run_recipe(root: Path, recipe: str, extra: list[str]) -> int:
    started = time.monotonic()
    exit_code = 1
    logged_argv = extra
    try:
        command, mode = command_for(root, recipe, extra)
        logged_argv = command or []
        if mode == "list":
            exit_code = print_catalog(load_catalog(root))
        elif mode == "report":
            exit_code = print_report(root)
        elif mode == "review":
            print_report(root)
            reset_review(root)
            print("command-review baseline reset to HEAD")
            exit_code = 0
        else:
            assert command is not None
            exit_code = subprocess.run(command, cwd=root, check=False).returncode
    except (KeyError, OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"command failed: {error}", file=sys.stderr)
        exit_code = 127
    finally:
        append_log(
            root,
            {
                "timestamp": datetime.now(UTC).isoformat(),
                "recipe": recipe,
                "argv": logged_argv,
                "duration_seconds": round(time.monotonic() - started, 6),
                "exit_code": exit_code,
            },
        )
    return exit_code


def main() -> int:
    if len(sys.argv) == 2 and sys.argv[1] == "--remind":
        return remind()
    if len(sys.argv) < 3 or sys.argv[1] != "run":
        print("usage: command_runner.py run RECIPE [ARGS...] | --remind", file=sys.stderr)
        return 2
    return run_recipe(ROOT, sys.argv[2], sys.argv[3:])


if __name__ == "__main__":
    raise SystemExit(main())
