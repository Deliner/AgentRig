import subprocess
from pathlib import Path

import pytest

from Project.project_memory.tests.consumer import committed_memory
from Project.shared.testing.consumer import invoke
from Project.shared.testing.repository import commit, revision


@pytest.mark.parametrize("damage", ["identity", "detail", "removal"])
def test_revision_memory_checks_parent_history_instead_of_candidate_or_checkout(
    worker: Path, tmp_path: Path, damage: str
) -> None:
    notes = committed_memory(worker, tmp_path)
    base = revision(tmp_path)
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
    candidate = commit(tmp_path)
    target.write_text(original)
    result = invoke(worker, tmp_path, "check", "--revision", candidate)
    assert result.returncode == 2, result.stdout + result.stderr
    assert "committed decision" in result.stderr
    assert target.read_text() == original
    result = invoke(worker, tmp_path, "check", "--revision", base)
    assert result.returncode == 0, result.stderr


def test_revision_memory_checks_the_second_merge_parent(worker: Path, tmp_path: Path) -> None:
    notes = committed_memory(worker, tmp_path)
    base = revision(tmp_path)
    source = tmp_path / "src/lib.rs"
    source.write_text(source.read_text() + "\n// left branch\n")
    left = commit(tmp_path)
    subprocess.run(["git", "checkout", "-q", base], cwd=tmp_path, capture_output=True, check=True)
    index = notes / "Decisions.md"
    index.write_text(index.read_text().replace("| Choice |", "| Changed |"))
    commit(tmp_path)
    subprocess.run(
        ["git", "merge", "--no-commit", left], cwd=tmp_path, capture_output=True, check=True
    )
    candidate = commit(tmp_path)
    result = invoke(worker, tmp_path, "check", "--revision", candidate)
    assert result.returncode == 2, result.stdout + result.stderr
    assert "committed decision identity cannot change" in result.stderr
