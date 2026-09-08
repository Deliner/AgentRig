import subprocess
from pathlib import Path

import pytest

from tooling.worker.src.scaffold.memory.tests.consumer import committed_memory
from tooling.worker.src.scaffold.testing.consumer import (
    invoke,
)
from tooling.worker.src.scaffold.testing.repository import commit, revision


@pytest.mark.parametrize("vcs", ["git", "hg", "private"])
@pytest.mark.parametrize("damage", ["identity", "detail", "removal"])
def test_revision_memory_checks_parent_history_instead_of_candidate_or_checkout(
    worker: Path, tmp_path: Path, vcs: str, damage: str
) -> None:
    notes = committed_memory(worker, tmp_path, vcs)
    base = revision(tmp_path, vcs)
    result = invoke(worker, tmp_path, "check", "--revision", base)
    assert result.returncode == 0, result.stderr
    index = notes / "Decisions.md"
    detail = notes / "Decisions/001.md"
    changes = {
        "identity": (index, index.read_text().replace("| Choice |", "| Changed |")),
        "detail": (detail, detail.read_text().replace("Text.", "Changed.")),
        "removal": (index, "# Decisions\n\n| ID | Decision | Applies in |\n"),
    }
    target, content = changes[damage]
    original = target.read_text()
    target.write_text(content)
    candidate = commit(tmp_path, vcs)
    target.write_text(original)
    result = invoke(worker, tmp_path, "check", "--revision", candidate)
    assert result.returncode == 2, result.stdout + result.stderr
    assert "committed decision" in result.stderr
    assert target.read_text() == original
    result = invoke(worker, tmp_path, "check", "--revision", base)
    assert result.returncode == 0, result.stderr


@pytest.mark.parametrize("vcs", ["git", "hg"])
def test_revision_memory_checks_the_second_merge_parent(
    worker: Path, tmp_path: Path, vcs: str
) -> None:
    notes = committed_memory(worker, tmp_path, vcs)
    base = revision(tmp_path, vcs)
    source = tmp_path / "src/lib.rs"
    source.write_text(source.read_text() + "\n// left branch\n")
    left = commit(tmp_path, vcs)
    using_git = vcs == "git"
    switch = ["checkout", "-q", base] if using_git else ["update", "--clean", "--rev", base]
    subprocess.run([vcs, *switch], cwd=tmp_path, capture_output=True, check=True)
    index = notes / "Decisions.md"
    index.write_text(index.read_text().replace("| Choice |", "| Changed |"))
    commit(tmp_path, vcs)
    merge = ["merge", "--no-commit", left] if using_git else ["merge", "--rev", left]
    subprocess.run([vcs, *merge], cwd=tmp_path, capture_output=True, check=True)
    candidate = commit(tmp_path, vcs)
    result = invoke(worker, tmp_path, "check", "--revision", candidate)
    assert result.returncode == 2, result.stdout + result.stderr
    assert "committed decision identity cannot change" in result.stderr
