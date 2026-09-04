#!/usr/bin/env python3

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path
from typing import Any

# DECISION: D003


def git(root: Path, *args: str) -> str:
    return subprocess.run(
        ["git", *args], cwd=root, check=True, text=True, stdout=subprocess.PIPE
    ).stdout


def root_for(event: dict[str, Any]) -> Path:
    cwd = Path(str(event.get("cwd") or Path.cwd()))
    return Path(git(cwd, "rev-parse", "--show-toplevel").strip())


def state_path(root: Path) -> Path:
    return root / ".git" / "codex-edit-checkpoint.json"


def failure_path(root: Path) -> Path:
    return root / ".git" / "codex-commit-failed"


def baseline_path(root: Path) -> Path:
    return root / ".git" / "codex-edit-baseline.json"


def snapshot(root: Path) -> dict[str, str]:
    output = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z"],
        cwd=root,
        check=True,
        stdout=subprocess.PIPE,
    ).stdout
    paths = [item.decode() for item in output.split(b"\0") if item]
    return {
        path: hashlib.sha256((root / path).read_bytes()).hexdigest()
        for path in paths
        if (root / path).is_file()
    }


def save_baseline(root: Path) -> None:
    baseline_path(root).write_text(json.dumps(snapshot(root)) + "\n", encoding="utf-8")


def targets_committed(root: Path, targets: dict[str, str | None]) -> bool:
    for path, expected in targets.items():
        result = subprocess.run(
            ["git", "show", f"HEAD:{path}"], cwd=root, check=False, stdout=subprocess.PIPE
        )
        actual = hashlib.sha256(result.stdout).hexdigest() if result.returncode == 0 else None
        if actual != expected:
            return False
    return True


def post_edit(root: Path) -> dict[str, Any] | None:
    before_path = baseline_path(root)
    before = json.loads(before_path.read_text(encoding="utf-8")) if before_path.exists() else {}
    before_path.unlink(missing_ok=True)
    after = snapshot(root)
    changed = {
        path: after.get(path)
        for path in before.keys() | after.keys()
        if before.get(path) != after.get(path)
    }
    if not changed:
        return None
    failure_path(root).unlink(missing_ok=True)
    head = git(root, "rev-parse", "HEAD").strip()
    checkpoint = state_path(root)
    prior = json.loads(checkpoint.read_text(encoding="utf-8")) if checkpoint.exists() else {}
    targets = prior.get("targets", {}) if prior.get("head") == head else {}
    targets.update(changed)
    state_path(root).write_text(
        json.dumps({"head": head, "targets": targets}, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    status = git(root, "status", "--short")
    summary = git(root, "diff", "--stat", "HEAD").strip() or status.strip()
    return {
        "systemMessage": "Edit checkpoint created; commit before another edit.",
        "hookSpecificOutput": {
            "hookEventName": "PostToolUse",
            "additionalContext": f"Commit checkpoint active. Current diff:\n{summary}",
        },
    }


def pre_edit(root: Path) -> dict[str, Any] | None:
    path = state_path(root)
    if not path.exists():
        failure_path(root).unlink(missing_ok=True)
        save_baseline(root)
        return None
    state = json.loads(path.read_text(encoding="utf-8"))
    head = git(root, "rev-parse", "HEAD").strip()
    if targets_committed(root, state.get("targets", {})):
        path.unlink(missing_ok=True)
        failure_path(root).unlink(missing_ok=True)
        save_baseline(root)
        return None
    failed = failure_path(root)
    if failed.exists():
        save_baseline(root)
        return None
    if state.get("head") != head:
        return {
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": "The checkpointed diff is not fully committed.",
            }
        }
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": "Commit the current checkpoint before another edit.",
        }
    }


def main() -> int:
    try:
        event = json.load(sys.stdin)
        if not isinstance(event, dict):
            return 0
        root = root_for(event)
        name = event.get("hook_event_name")
        result = pre_edit(root) if name == "PreToolUse" else post_edit(root)
        if result is not None:
            json.dump(result, sys.stdout)
            sys.stdout.write("\n")
    except Exception as error:
        print(f"commit checkpoint hook: {error}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
