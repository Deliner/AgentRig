import json
import subprocess
from pathlib import Path

import pytest

from tooling.worker.src.scaffold.testing.consumer import (
    CONFIG,
    file_contents,
    git,
    invoke,
    memory,
    project,
    update_config,
    vcs_backend,
)
from tooling.worker.src.scaffold.testing.repository import (
    GATE,
    commit,
    repository,
    resumed,
    revision,
)


@pytest.mark.parametrize("vcs", ["git", "hg"])
def test_resume_resolves_state_revision(worker: Path, tmp_path: Path, vcs: str) -> None:
    repository(tmp_path, vcs)
    saved = revision(tmp_path, vcs)
    using_git = vcs == "git"
    branch = "trunk" if using_git else "default"
    state = tmp_path / "notes/State.md"
    state.write_text(state.read_text() + f"\nBranch: `{branch}`\nRevision: `{saved[:12]}`\n")
    value = resumed(worker, tmp_path)
    assert value["snapshot"] == "current"
    assert value["state_revision"]["head_changed"] is False
    assert value["vcs"]["revision"] == saved
    assert value["vcs"]["branch"] == branch
    assert not value["vcs"]["merge_in_progress"]
    assert not value["vcs"]["rebase_in_progress"]
    commit(tmp_path, vcs)
    value = resumed(worker, tmp_path)
    assert value["snapshot"] == "stale"
    assert value["state_revision"]["head_changed"] is True


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
    assert value["git"][f"{operation}_in_progress"]
    assert file_contents(tmp_path) == before


@pytest.mark.parametrize("operation", ["merge", "rebase"])
def test_mercurial_resume_observes_real_conflict(
    worker: Path, tmp_path: Path, operation: str
) -> None:
    repository(tmp_path, "hg")
    source = tmp_path / "src/value.py"
    source.write_text("value = 2\n")
    feature = commit(tmp_path, "hg")
    subprocess.run(["hg", "update", "-r", "0"], cwd=tmp_path, capture_output=True, check=True)
    source.write_text("value = 3\n")
    base = commit(tmp_path, "hg")
    subprocess.run(["hg", "update", "-r", feature], cwd=tmp_path, capture_output=True, check=True)
    merging = operation == "merge"
    args = (
        ["merge", "-r", base]
        if merging
        else ["--config", "extensions.rebase=", "rebase", "-s", feature, "-d", base]
    )
    result = subprocess.run(
        ["hg", *args, "--tool", "internal:fail"], cwd=tmp_path, capture_output=True, text=True
    )
    assert result.returncode != 0, result.stdout + result.stderr
    status = subprocess.check_output(["hg", "status"], cwd=tmp_path, text=True).strip()
    before = file_contents(tmp_path)
    value = resumed(worker, tmp_path)
    assert value["vcs"]["backend"] == "mercurial"
    assert value["vcs"][f"{operation}_in_progress"]
    assert value["vcs"]["status"] == status
    assert "git" not in value
    assert file_contents(tmp_path) == before


@pytest.mark.parametrize("vcs", ["git", "hg"])
def test_resume_distinguishes_plain_and_unborn_repositories(
    worker: Path, tmp_path: Path, vcs: str
) -> None:
    project(tmp_path, CONFIG + GATE)
    update_config(tmp_path / "agentrig.yaml", git={"backend": vcs_backend(vcs)})
    memory(tmp_path)
    assert resumed(worker, tmp_path)["vcs"] is None
    subprocess.run([vcs, "init"], cwd=tmp_path, capture_output=True, check=True)
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
    assert value["git"] == value["vcs"]
    assert file_contents(linked) == before


def test_private_resume_accepts_opaque_recorded_revisions(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, CONFIG + GATE)
    notes = memory(tmp_path)
    state = notes / "State.md"
    state.write_text(state.read_text() + "\nBranch: team/main\n\nRevision: revision-42\n")
    observed = {
        "branch": "team/main",
        "revision": "revision-42",
        "status": "",
        "merge_in_progress": False,
        "rebase_in_progress": False,
    }
    replies = {"observe": observed, "resolve": "revision-42"}
    script = f"import json,sys; r=json.load(sys.stdin); print(json.dumps({{'version':1,'result':{replies!r}[r['operation']]}}))"
    adapter = {"command": ["python3", "-c", script]}
    update_config(tmp_path / "agentrig.yaml", git={"backend": adapter})
    result = invoke(worker, tmp_path, "resume")
    assert result.returncode == 0, result.stderr
    report = json.loads(result.stdout)
    assert report["vcs"]["backend"] == adapter
    assert report["snapshot"] == "current"
    assert report["state_revision"]["resolved"] == "revision-42"
    assert "git" not in report
