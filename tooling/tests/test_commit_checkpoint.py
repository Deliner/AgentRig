from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

# DECISION: D003
# DECISION: D011

ROOT = Path(__file__).parents[2]


def git(root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], cwd=root, check=False, text=True, capture_output=True)


def checked_git(root: Path, *args: str) -> str:
    result = git(root, *args)
    assert result.returncode == 0, result.stderr
    return result.stdout.strip()


# INVARIANT: I006
def test_vac_commit_boundary(tmp_path: Path) -> None:
    checked_git(tmp_path, "init", "-q", "-b", "feature/vac")
    checked_git(tmp_path, "config", "user.name", "Test")
    checked_git(tmp_path, "config", "user.email", "test@example.invalid")
    source = tmp_path / "source.txt"
    source.write_text("base\n", encoding="utf-8")
    checked_git(tmp_path, "add", "source.txt")
    checked_git(tmp_path, "commit", "-qm", "base")
    baseline = checked_git(tmp_path, "rev-parse", "HEAD")
    hooks = tmp_path / ".githooks"
    hooks.mkdir()
    shutil.copy(ROOT / ".githooks/pre-commit", hooks / "pre-commit")
    (hooks / "pre-commit").chmod(0o755)
    tooling = tmp_path / "tooling"
    tooling.mkdir()
    shutil.copy(ROOT / "tooling/branch_workflow.py", tooling / "branch_workflow.py")
    gate = tooling / "check.sh"
    gate.write_text(
        '#!/usr/bin/env bash\nset -euo pipefail\n[[ "$1" == "--staged" ]]\n'
        '[[ "$(git show :source.txt)" == "verified" ]]\n',
        encoding="utf-8",
    )
    gate.chmod(0o755)
    checked_git(tmp_path, "config", "core.hooksPath", ".githooks")
    unrelated = tmp_path / "unrelated.txt"
    unrelated.write_text("user work\n", encoding="utf-8")
    source.write_text("first edit\n", encoding="utf-8")
    source.write_text("second edit\n", encoding="utf-8")
    checked_git(tmp_path, "add", "source.txt")
    source.write_text("verified\n", encoding="utf-8")
    assert git(tmp_path, "commit", "-qm", "invalid staged content").returncode != 0
    assert checked_git(tmp_path, "rev-parse", "HEAD") == baseline
    assert not (tmp_path / ".git/codex-commit-failed").exists()
    checked_git(tmp_path, "add", "source.txt")
    source.write_text("unstaged later work\n", encoding="utf-8")
    checked_git(tmp_path, "commit", "-qm", "verified VAC")
    assert checked_git(tmp_path, "show", "HEAD:source.txt") == "verified"
    assert source.read_text(encoding="utf-8") == "unstaged later work\n"
    assert git(tmp_path, "cat-file", "-e", "HEAD:unrelated.txt").returncode != 0
    assert unrelated.read_text(encoding="utf-8") == "user work\n"
    checked_git(tmp_path, "switch", "-c", "master")
    source.write_text("verified\n", encoding="utf-8")
    assert git(tmp_path, "commit", "--allow-empty", "-qm", "direct master").returncode != 0


@pytest.mark.parametrize("event", ["PreToolUse", "PostToolUse"])
def test_retired_checkpoint_does_not_block_or_mutate(tmp_path: Path, event: str) -> None:
    state = tmp_path / ".git"
    state.mkdir()
    checkpoint = state / "codex-edit-checkpoint.json"
    checkpoint.write_text("old or malformed state", encoding="utf-8")
    for _ in range(2):
        result = subprocess.run(
            [sys.executable, str(ROOT / ".codex/hooks/commit_checkpoint.py")],
            cwd=tmp_path,
            input=json.dumps({"hook_event_name": event, "cwd": str(tmp_path)}),
            capture_output=True,
            text=True,
            check=False,
        )
        assert result.returncode == 0
        assert result.stdout == result.stderr == ""
    assert checkpoint.read_text(encoding="utf-8") == "old or malformed state"
    assert list(state.iterdir()) == [checkpoint]


def test_edit_hooks_keep_reminders_without_checkpoint() -> None:
    configuration = json.loads((ROOT / ".codex/hooks.json").read_text(encoding="utf-8"))
    hooks = configuration["hooks"]
    assert "commit_checkpoint" not in json.dumps(hooks)
    edits = [item for item in hooks["PreToolUse"] if item["matcher"] == "apply_patch|Edit|Write"]
    assert len(edits) == 1
    assert len(edits[0]["hooks"]) == 1
    assert "complexity_discipline_reminder.py" in edits[0]["hooks"][0]["command"]
