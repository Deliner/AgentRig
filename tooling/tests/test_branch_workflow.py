from __future__ import annotations

import os
import subprocess
from pathlib import Path

from branch_workflow import (
    ZERO,
    commit_allowed,
    current_branch,
    is_ancestor,
    merge_feature,
    reference_allowed,
    start_feature,
)

# DECISION: D010


def git(root: Path, *args: str) -> str:
    return subprocess.run(
        ["git", *args],
        cwd=root,
        check=True,
        text=True,
        capture_output=True,
    ).stdout.strip()


def commit(root: Path, message: str) -> None:
    git(root, "add", ".")
    git(root, "commit", "-m", message)


def repository(root: Path) -> Path:
    git(root, "init", "-q", "-b", "master")
    git(root, "config", "user.name", "Test")
    git(root, "config", "user.email", "test@example.invalid")
    tooling = root / "tooling"
    tooling.mkdir()
    check = tooling / "check.sh"
    line_break = chr(10)
    check.write_text(line_break.join(["#!/usr/bin/env bash", "exit 0", ""]), encoding="utf-8")
    os.chmod(check, 0o755)
    (root / "base.txt").write_text("base", encoding="utf-8")
    commit(root, "base")
    return root


# INVARIANT: I010
def test_feature_branch_policy(tmp_path: Path) -> None:
    root = repository(tmp_path)
    assert not commit_allowed(root)
    assert start_feature(root, "example") == 0
    assert current_branch(root) == "feature/example"
    assert commit_allowed(root)
    (root / "feature.txt").write_text("feature", encoding="utf-8")
    commit(root, "feature")

    git(root, "switch", "master")
    (root / "master.txt").write_text("master", encoding="utf-8")
    commit(root, "concurrent master")
    current_master = git(root, "rev-parse", "master")
    git(root, "switch", "feature/example")

    assert merge_feature(root) == 0
    assert current_branch(root) == "master"
    feature_tip = git(root, "rev-parse", "feature/example")
    assert git(root, "rev-parse", "feature/example^") == current_master
    assert is_ancestor(root, feature_tip, "master")
    parents = git(root, "rev-list", "--parents", "-1", "master").split()
    assert parents[1:] == [current_master, feature_tip]
    assert (root / "feature.txt").is_file()
    assert (root / "master.txt").is_file()
    deletion = f"{feature_tip} {ZERO} refs/heads/feature/example"
    assert not reference_allowed(root, [deletion])
