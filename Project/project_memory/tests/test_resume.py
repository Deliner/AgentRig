import subprocess
from pathlib import Path

import pytest

from Project.shared.testing.consumer import (
    CONFIG,
    file_contents,
    git,
    memory,
    project,
)
from Project.shared.testing.repository import (
    GATE,
    commit,
    repository,
    resumed,
    revision,
)


def test_resume_resolves_state_revision(worker: Path, tmp_path: Path) -> None:
    repository(tmp_path)
    saved = revision(tmp_path)
    state = tmp_path / "notes/State.md"
    state.write_text(state.read_text() + f"\nBranch: `trunk`\nRevision: `{saved[:12]}`\n")
    value = resumed(worker, tmp_path)
    assert value["snapshot"] == "current"
    assert value["state_revision"]["head_changed"] is False
    assert value["vcs"]["revision"] == saved
    assert value["vcs"]["branch"] == "trunk"
    assert not value["vcs"]["merge_in_progress"]
    assert not value["vcs"]["rebase_in_progress"]
    commit(tmp_path)
    value = resumed(worker, tmp_path)
    assert value["snapshot"] == "stale"
    assert value["state_revision"]["head_changed"] is True


def test_resume_does_not_verify_branch_only_state(worker: Path, tmp_path: Path) -> None:
    repository(tmp_path)
    initial = resumed(worker, tmp_path)
    state = tmp_path / "notes/State.md"
    state.write_text(state.read_text() + f"\nBranch: {initial['vcs']['branch']}\n")
    assert resumed(worker, tmp_path)["snapshot"] == "unverified"
    commit(tmp_path)
    result = resumed(worker, tmp_path)
    assert result["vcs"]["revision"] != initial["vcs"]["revision"]
    assert result["snapshot"] == "unverified"
    assert result["state_revision"]["head_changed"] is None
    state.write_text(state.read_text().replace("Branch: ", "Branch: another-"))
    assert resumed(worker, tmp_path)["snapshot"] == "stale"


@pytest.mark.parametrize("operation", ["merge", "rebase"])
# INVARIANT: I019
def test_resume_observes_real_conflict(worker: Path, tmp_path: Path, operation: str) -> None:
    repository(tmp_path)
    source = tmp_path / "src/value.py"
    git(tmp_path, "switch", "-qc", "task/conflict")
    source.write_text("value = 2\n")
    git(tmp_path, "commit", "-am", "feature")
    git(tmp_path, "switch", "trunk")
    source.write_text("value = 3\n")
    git(tmp_path, "commit", "-am", "base")
    git(tmp_path, "switch", "task/conflict")
    assert git(tmp_path, operation, "trunk", success=False).returncode != 0
    before = file_contents(tmp_path)
    value = resumed(worker, tmp_path)
    assert value["vcs"][f"{operation}_in_progress"]
    assert file_contents(tmp_path) == before


def test_resume_distinguishes_plain_and_unborn_repositories(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, CONFIG + GATE)
    memory(tmp_path)
    assert resumed(worker, tmp_path)["vcs"] is None
    subprocess.run(["git", "init"], cwd=tmp_path, capture_output=True, check=True)
    before = file_contents(tmp_path)
    value = resumed(worker, tmp_path)
    assert value["vcs"]["revision"] == ""
    assert not value["vcs"]["merge_in_progress"]
    assert not value["vcs"]["rebase_in_progress"]
    assert file_contents(tmp_path) == before


def test_resume_observes_linked_git_worktree(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.delenv("GIT_INDEX_FILE", raising=False)
    repository(tmp_path)
    linked = tmp_path.parent / f"{tmp_path.name}-linked"
    git(tmp_path, "worktree", "add", "-b", "task/linked", str(linked))
    before = file_contents(linked)
    value = resumed(worker, linked)
    assert value["vcs"]["branch"] == "task/linked"
    assert file_contents(linked) == before
