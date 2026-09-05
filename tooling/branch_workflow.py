#!/usr/bin/env python3

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

# DECISION: D010

MASTER = "master"
FEATURE_PREFIX = "feature/"
FEATURE_NAME = re.compile(r"^[a-z0-9][a-z0-9._-]*$")


def git(root: Path, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        cwd=root,
        check=check,
        text=True,
        capture_output=True,
    )


def current_branch(root: Path) -> str:
    return git(root, "branch", "--show-current").stdout.strip()


def require_clean(root: Path) -> None:
    if git(root, "status", "--porcelain").stdout:
        raise ValueError("working tree must be clean")


def is_ancestor(root: Path, ancestor: str, descendant: str) -> bool:
    return (
        git(root, "merge-base", "--is-ancestor", ancestor, descendant, check=False).returncode == 0
    )


def start_feature(root: Path, name: str) -> int:
    if current_branch(root) != MASTER:
        raise ValueError("feature branches must start from master")
    require_clean(root)
    if FEATURE_NAME.fullmatch(name) is None:
        raise ValueError(
            "feature name must use lowercase letters, digits, dot, dash, or underscore"
        )
    branch = f"{FEATURE_PREFIX}{name}"
    if (
        git(root, "show-ref", "--verify", "--quiet", f"refs/heads/{branch}", check=False).returncode
        == 0
    ):
        raise ValueError(f"branch already exists: {branch}")
    git(root, "switch", "-c", branch)
    print(branch)
    return 0


def merge_feature(root: Path) -> int:
    feature = current_branch(root)
    if not feature.startswith(FEATURE_PREFIX):
        raise ValueError("feature-merge must run from a feature/* branch")
    require_clean(root)
    if not is_ancestor(root, MASTER, feature):
        result = git(root, "rebase", "--rebase-merges", MASTER, check=False)
        if result.returncode != 0:
            sys.stderr.write(result.stdout + result.stderr)
            return result.returncode
    verification = subprocess.run([str(root / "tooling" / "check.sh")], cwd=root, check=False)
    if verification.returncode != 0:
        return verification.returncode
    master_before = git(root, "rev-parse", MASTER).stdout.strip()
    git(root, "switch", MASTER)
    if git(root, "rev-parse", MASTER).stdout.strip() != master_before:
        git(root, "switch", feature)
        raise ValueError("master changed during integration; retry feature-merge")
    if not is_ancestor(root, MASTER, feature):
        git(root, "switch", feature)
        raise ValueError("feature is not based on current master; retry feature-merge")
    result = git(root, "merge", "--no-ff", "--no-edit", feature, check=False)
    if result.returncode != 0:
        sys.stderr.write(result.stdout + result.stderr)
        return result.returncode
    print(f"merged {feature}; branch retained")
    return 0


def repository_root() -> Path:
    result = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        check=True,
        text=True,
        capture_output=True,
    )
    return Path(result.stdout.strip())


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: branch_workflow.py COMMAND [ARGS]", file=sys.stderr)
        return 2
    root = repository_root()
    command, arguments = sys.argv[1], sys.argv[2:]
    try:
        if command == "start" and len(arguments) == 1:
            return start_feature(root, arguments[0])
        if command == "merge" and not arguments:
            return merge_feature(root)
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"branch workflow: {error}", file=sys.stderr)
        return 1
    print("invalid branch workflow arguments", file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
