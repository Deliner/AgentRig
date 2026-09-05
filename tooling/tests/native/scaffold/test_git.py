import json
import subprocess
from pathlib import Path

import pytest
from support import invoke


def git(root: Path, *args: str) -> str:
    result = subprocess.run(["git", *args], cwd=root, capture_output=True, text=True, check=False)
    assert result.returncode == 0, result.stdout + result.stderr
    return result.stdout.strip()


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


def test_divergent_rebase_preserves_merge_history(worker: Path, installed: Path) -> None:
    root = installed
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
