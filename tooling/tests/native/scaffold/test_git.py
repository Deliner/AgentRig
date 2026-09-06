# DECISION: D014
# DECISION: D012
# DECISION: D011
# DECISION: D010
# DECISION: D003
import json
import shlex
import subprocess
from pathlib import Path

import pytest
from support import git as git_result
from support import invoke, update_config
from test_commands import background_id, require_user_systemd, wait_for_background_output
from test_feedback import commit as revision_commit
from test_feedback import evidence, repository, revision


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
    config = tmp_path / "agentrig.yaml"
    update_config(
        config,
        commands={"probe": {"argv": ["sh", "-c", 'test -z "$BLOCK_DELIVERY"']}},
        checks=[
            {
                "id": "probe",
                "kind": "command",
                "command": "probe",
                "skill": ".agentrig/skills/repair/SKILL.md",
            }
        ],
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
    argv = ["python3", "-c", "import time; print('ready',flush=True); time.sleep(60)"]
    config = root / "agentrig.yaml"
    update_config(
        config,
        commands={"wait": {"argv": argv}, "service": {"argv": argv.copy(), "lifetime": "shared"}},
    )


def mercurial_gate(worker: Path, root: Path, command: str = "exit 0") -> None:
    repository(root, "hg")
    config = root / "agentrig.yaml"
    config.write_text(config.read_text().replace("exit 23", command))
    (root / "src/other.py").write_text("other = 1\n")
    subprocess.run(["hg", "add", "src/other.py"], cwd=root, check=True, capture_output=True)
    revision_commit(root, "hg")
    (root / ".hg/hgrc").write_text(
        "[hooks]\npretxncommit.agentrig = "
        + shlex.quote(str(worker))
        + " check --root "
        + shlex.quote(str(root))
        + ' --revision "$HG_NODE"\n'
    )


def hg_commit(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["hg", "commit", "-m", "candidate", "-u", "Test", "src/value.py"],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
        timeout=30,
    )


def test_mercurial_transaction_gate_rolls_back_failures_and_preserves_unselected_work(
    worker: Path, tmp_path: Path
) -> None:
    mercurial_gate(worker, tmp_path)
    base = revision(tmp_path, "hg")
    source = tmp_path / "src/value.py"
    source.write_text("line\n" * 61)
    unrelated = tmp_path / "src/other.py"
    unrelated.write_text("unselected\n" * 61)
    failed = hg_commit(tmp_path)
    assert failed.returncode != 0, failed.stdout + failed.stderr
    assert "exceeds 60" in failed.stdout
    assert revision(tmp_path, "hg") == base
    assert evidence(tmp_path)["code"] == 1
    assert source.read_text() == "line\n" * 61
    source.write_text("value = 2\n")
    passed = hg_commit(tmp_path)
    assert passed.returncode == 0, passed.stdout + passed.stderr
    committed = revision(tmp_path, "hg")
    assert committed != base
    record = evidence(tmp_path)
    assert record["revision"] == committed and record["status"] == "completed"
    assert record["code"] == 0 and record["only"] is None
    assert unrelated.read_text() == "unselected\n" * 61
    selected = subprocess.check_output(["hg", "cat", "-r", committed, "src/other.py"], cwd=tmp_path)
    assert selected == b"other = 1\n"


def test_mercurial_transaction_rejects_a_check_that_mutates_its_export(
    worker: Path, tmp_path: Path
) -> None:
    mercurial_gate(worker, tmp_path, "echo changed > src/value.py")
    base = revision(tmp_path, "hg")
    source = tmp_path / "src/value.py"
    source.write_text("value = 2\n")
    result = hg_commit(tmp_path)
    assert result.returncode != 0, result.stdout + result.stderr
    assert revision(tmp_path, "hg") == base
    assert evidence(tmp_path)["status"] == "inputs-changed"
    assert source.read_text() == "value = 2\n"


def hg(root: Path, *args: str) -> str:
    return subprocess.check_output(["hg", *args], cwd=root, text=True).strip()


def test_mercurial_setup_registers_native_hooks_and_checks_commits(
    worker: Path, tmp_path: Path
) -> None:
    initialize_mercurial(worker, tmp_path)
    preview = invoke(worker, tmp_path, "setup", "--preview")
    assert preview.returncode == 0, preview.stderr
    assert json.loads(preview.stdout)["registrations"]["vcs"]["backend"] == "mercurial"
    assert not (tmp_path / ".hg").exists() and not (tmp_path / ".git").exists()
    result = invoke(worker, tmp_path, "setup")
    assert result.returncode == 0, result.stdout + result.stderr
    assert hg(tmp_path, "branch") == "trunk"
    assert "pretxncommit.agentrig" in (tmp_path / ".hg/hgrc").read_text()
    assert "hg root" in (tmp_path / ".codex/hooks.json").read_text()
    assert not (tmp_path / ".git").exists()
    (tmp_path / "src").mkdir()
    (tmp_path / "src/test_sample.py").write_text("def test_value():\n    assert 1 == 1\n")
    hg(tmp_path, "add")
    denied = subprocess.run(
        ["hg", "commit", "-m", "base", "-u", "Test"], cwd=tmp_path, capture_output=True, text=True
    )
    assert denied.returncode != 0 and "direct commits" in denied.stderr
    hg(tmp_path, "branch", "task/bootstrap")
    hg(tmp_path, "commit", "-m", "bootstrap", "-u", "Test")
    assert invoke(worker, tmp_path, "doctor").returncode == 0
    assert "runtime" not in hg(tmp_path, "status", "--unknown")
    before = (tmp_path / ".hg/hgrc").read_bytes()
    assert invoke(worker, tmp_path, "setup").returncode == 0
    assert (tmp_path / ".hg/hgrc").read_bytes() == before


def initialize_mercurial(worker: Path, root: Path) -> None:
    result = invoke(
        worker,
        root,
        "init",
        "--vcs",
        "mercurial",
        "--base",
        "trunk",
        "--prefix",
        "task/",
        "--service",
        "rig space",
    )
    assert result.returncode == 0, result.stderr


def test_mercurial_setup_preserves_custom_hook_registration(worker: Path, tmp_path: Path) -> None:
    hg(tmp_path, "init")
    hgrc = tmp_path / ".hg/hgrc"
    existing = "[ui]\nusername = Test\n[hooks]\npretxncommit.custom = true\n"
    hgrc.write_text(existing)
    ignore = tmp_path / ".hgignore"
    ignore.write_text("syntax: glob\nuser-generated/**\n")
    assert invoke(worker, tmp_path, "init", "--vcs", "mercurial").returncode == 0
    assert hgrc.read_text().startswith(existing)
    assert ignore.read_text() == "syntax: glob\nuser-generated/**\n"
    assert hg(tmp_path, "config", "hooks.pretxncommit.custom") == "true"
    with hgrc.open("a") as stream:
        stream.write("\n[hooks]\npretxncommit.agentrig = false\n")
    before = hgrc.read_bytes()
    result = invoke(worker, tmp_path, "setup", "--preview")
    assert result.returncode == 2 and "existing registration preserved" in result.stderr
    assert hgrc.read_bytes() == before


def test_mercurial_review_defaults_select_the_native_repository(
    worker: Path, tmp_path: Path
) -> None:
    result = invoke(worker, tmp_path, "init", "--vcs", "mercurial", "--review", "true")
    assert result.returncode == 0, result.stderr
    for name in ["code", "research"]:
        source = (tmp_path / f".agentrig/review/config/projects/{name}.yaml").read_text()
        assert "vcs: mercurial" in source
    result = invoke(worker, tmp_path, "review", "config-check")
    assert result.returncode == 0, result.stderr


def test_vcs_selection_preserves_legacy_git_and_rejects_conflicts(
    worker: Path, tmp_path: Path
) -> None:
    assert invoke(worker, tmp_path, "init").returncode == 0
    path = tmp_path / "agentrig.yaml"
    canonical = path.read_text()
    assert "vcs:" in canonical
    path.write_text(canonical.replace("vcs:", "git:").replace("  backend: git\n", ""))
    assert invoke(worker, tmp_path, "config-check").returncode == 0
    path.write_text(canonical + "\ngit:\n  base: other\n  prefix: task/\n")
    assert invoke(worker, tmp_path, "config-check").returncode == 2
    path.write_text(canonical.replace("backend: git", "backend: unknown"))
    assert invoke(worker, tmp_path, "config-check").returncode == 2
    path.write_text(canonical)
    hg(tmp_path, "init")
    result = invoke(worker, tmp_path, "setup")
    assert result.returncode == 2 and "does not match" in result.stderr
    assert not (tmp_path / ".git").exists()
