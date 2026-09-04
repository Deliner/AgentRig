from __future__ import annotations

import subprocess
from pathlib import Path

from commit_checkpoint import post_edit, pre_edit

# DECISION: D003


def commit(root: Path, message: str) -> None:
    subprocess.run(["git", "add", "."], cwd=root, check=True)
    subprocess.run(
        [
            "git",
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-qm",
            message,
        ],
        cwd=root,
        check=True,
    )


# INVARIANT: I006
def test_checkpoint_blocks_until_commit_and_preserves_unrelated_work(tmp_path: Path) -> None:
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    tracked = tmp_path / "tracked.txt"
    tracked.write_text("base\n", encoding="utf-8")
    commit(tmp_path, "base")
    unrelated = tmp_path / "unrelated.txt"
    unrelated.write_text("user work\n", encoding="utf-8")
    assert pre_edit(tmp_path) is None
    tracked.write_text("changed\n", encoding="utf-8")
    assert post_edit(tmp_path) is not None
    assert pre_edit(tmp_path) is not None
    subprocess.run(["git", "add", "tracked.txt"], cwd=tmp_path, check=True)
    subprocess.run(
        [
            "git",
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-qm",
            "change",
        ],
        cwd=tmp_path,
        check=True,
    )
    assert pre_edit(tmp_path) is None
    assert unrelated.read_text(encoding="utf-8") == "user work\n"
