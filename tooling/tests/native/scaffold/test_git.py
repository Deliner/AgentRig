# DECISION: D014
# DECISION: D012
# DECISION: D011
# DECISION: D010
# DECISION: D003
import json
import subprocess
from pathlib import Path

import pytest
from support import git as git_result
from support import invoke
from test_commands import background_id, require_user_systemd, wait_for_background_output


def git(root: Path, *args: str) -> str:
    return git_result(root, *args).stdout.strip()


def commit(root: Path, name: str, text: str) -> None:
    (root / name).write_text(text)
    git(root, "add", ".")
    git(root, "commit", "-qm", name)


@pytest.fixture
def installed(worker: Path, tmp_path: Path) -> Path:
    git(tmp_path, "init", "-q", "-b", "trunk")
    git(tmp_path, "config", "user.name", "Test")
    git(tmp_path, "config", "user.email", "test@example.invalid")
    commit(tmp_path, "shared.txt", "initial\n")
    git(tmp_path, "switch", "-c", "task/bootstrap")
    assert invoke(worker, tmp_path, "init", "--base", "trunk", "--prefix", "task/").returncode == 0
    config = tmp_path / "worker.toml"
    with config.open("a") as stream:
        argv = json.dumps(["sh", "-c", 'test -z "$BLOCK_DELIVERY"'])
        stream.write(
            f'\n[commands.probe]\nargv = {argv}\n[[checks]]\nid = "probe"\nkind = "command"\ncommand = "probe"\nskill = ".worker/skills/repair/SKILL.md"\n'
        )
    commit(tmp_path, "ready.txt", "ready")
    result = invoke(worker, tmp_path, "feature-merge")
    assert result.returncode == 0, result.stdout + result.stderr
    return tmp_path


# INVARIANT: I010
def test_divergent_rebase_preserves_merge_history(worker: Path, installed: Path) -> None:
    root = installed
    base_before = git(root, "rev-parse", "HEAD")
    denied = git_result(root, "commit", "--allow-empty", "-qm", "direct base commit", success=False)
    assert denied.returncode != 0
    assert git(root, "rev-parse", "HEAD") == base_before
    assert invoke(worker, root, "feature-start", "product").returncode == 0
    commit(root, "product.txt", "product")
    git(root, "switch", "-c", "task/product-side")
    commit(root, "side.txt", "side")
    git(root, "switch", "task/product")
    commit(root, "main.txt", "main")
    git(root, "merge", "--no-ff", "--no-edit", "task/product-side")
    git(root, "switch", "trunk")
    assert invoke(worker, root, "feature-start", "concurrent").returncode == 0
    commit(root, "concurrent.txt", "concurrent")
    assert invoke(worker, root, "feature-merge").returncode == 0
    base = git(root, "rev-parse", "trunk")
    git(root, "switch", "task/product")
    result = invoke(worker, root, "feature-merge")
    assert result.returncode == 0, result.stdout + result.stderr
    assert git(root, "branch", "--show-current") == "trunk"
    tip = git(root, "rev-parse", "task/product")
    assert len(git(root, "rev-list", "--parents", "-1", tip).split()) == 3
    assert git(root, "rev-list", "--parents", "-1", "trunk").split()[1:] == [base, tip]
    assert git(root, "status", "--porcelain") == ""
    for name in ["product", "side", "main", "concurrent"]:
        assert (root / f"{name}.txt").read_text() == name


def test_conflicting_rebase_can_be_aborted_without_losing_feature(
    worker: Path, installed: Path
) -> None:
    root = installed
    assert invoke(worker, root, "feature-start", "product").returncode == 0
    commit(root, "shared.txt", "product\n")
    product = git(root, "rev-parse", "HEAD")
    git(root, "switch", "trunk")
    assert invoke(worker, root, "feature-start", "concurrent").returncode == 0
    commit(root, "shared.txt", "concurrent\n")
    assert invoke(worker, root, "feature-merge").returncode == 0
    base = git(root, "rev-parse", "trunk")
    git(root, "switch", "task/product")
    result = invoke(worker, root, "feature-merge")
    assert result.returncode != 0
    assert "CONFLICT" in result.stdout + result.stderr
    assert git(root, "rev-parse", "trunk") == base
    assert git(root, "ls-files", "--unmerged")
    git(root, "rebase", "--abort")
    assert git(root, "rev-parse", "HEAD") == product
    assert (root / "shared.txt").read_text() == "product\n"
    assert git(root, "status", "--porcelain") == ""


def test_gate_failure_and_dirty_workspace_preserve_branches(
    worker: Path, installed: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    root = installed
    assert invoke(worker, root, "feature-start", "product").returncode == 0
    commit(root, "product.txt", "product")
    feature = git(root, "rev-parse", "HEAD")
    base = git(root, "rev-parse", "trunk")
    monkeypatch.setenv("BLOCK_DELIVERY", "1")
    result = invoke(worker, root, "feature-merge")
    assert result.returncode == 1
    assert "ERROR [probe]" in result.stderr and "repair/SKILL.md" in result.stderr
    assert git(root, "branch", "--show-current") == "task/product"
    assert git(root, "rev-parse", "HEAD") == feature
    assert git(root, "rev-parse", "trunk") == base
    monkeypatch.delenv("BLOCK_DELIVERY")
    (root / "product.txt").write_text("uncommitted")
    result = invoke(worker, root, "feature-merge")
    assert result.returncode == 2
    assert "working tree must be clean" in result.stderr
    assert (root / "product.txt").read_text() == "uncommitted"
    assert git(root, "rev-parse", "HEAD") == feature


# INVARIANT: I006
def test_staged_commit_preserves_unstaged_work(
    worker: Path, installed: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    root = installed
    assert invoke(worker, root, "feature-start", "atomic").returncode == 0
    before = git(root, "rev-parse", "HEAD")
    source = root / "product.txt"
    source.write_text("staged")
    git(root, "add", "product.txt")
    source.write_text("later unstaged work")
    unrelated = root / "unrelated.txt"
    unrelated.write_text("user work")
    monkeypatch.setenv("BLOCK_DELIVERY", "1")
    result = subprocess.run(
        ["git", "commit", "-qm", "blocked"], cwd=root, capture_output=True, check=False
    )
    assert result.returncode != 0
    assert git(root, "rev-parse", "HEAD") == before
    monkeypatch.delenv("BLOCK_DELIVERY")
    git(root, "commit", "-qm", "verified index")
    assert git(root, "show", "HEAD:product.txt") == "staged"
    assert source.read_text() == "later unstaged work"
    assert "unrelated.txt" not in git(root, "ls-tree", "--name-only", "HEAD").splitlines()
    assert unrelated.read_text() == "user work"


def test_merge_cleans_only_owned_task_runs(
    worker: Path, installed: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    require_user_systemd()
    root = installed
    assert invoke(worker, root, "feature-start", "jobs").returncode == 0
    background_commands(root)
    commit(root, "work.txt", "feature")
    monkeypatch.setenv("WORKER_OWNER", "merge-owner")
    owned = background_id(worker, root)
    shared = background_id(worker, root, "service")
    monkeypatch.setenv("WORKER_OWNER", "other-owner")
    other = background_id(worker, root)
    try:
        for identifier in [owned, shared, other]:
            wait_for_background_output(worker, root, identifier)
        monkeypatch.setenv("WORKER_OWNER", "merge-owner")
        result = invoke(worker, root, "feature-merge")
        assert result.returncode == 0, result.stdout + result.stderr
        assert git(root, "branch", "--show-current") == "trunk"
        assert json.loads(invoke(worker, root, "job-status", owned).stdout)["state"] == "cancelled"
        for identifier in [shared, other]:
            assert (
                json.loads(invoke(worker, root, "job-status", identifier).stdout)["state"]
                == "running"
            )
        assert "shared service" in result.stdout
    finally:
        for owner, identifier in [
            ("merge-owner", owned),
            ("merge-owner", shared),
            ("other-owner", other),
        ]:
            monkeypatch.setenv("WORKER_OWNER", owner)
            result = invoke(worker, root, "job-stop", identifier)
            assert result.returncode == 0, result.stderr


def background_commands(root: Path) -> None:
    argv = json.dumps(["python3", "-c", "import time; print('ready',flush=True); time.sleep(60)"])
    config = root / "worker.toml"
    config.write_text(
        config.read_text()
        + f'\n[commands.wait]\nargv = {argv}\n[commands.service]\nargv = {argv}\nlifetime = "shared"\n'
    )
