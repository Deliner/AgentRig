import subprocess
from pathlib import Path

import pytest

from tooling.worker.src.scaffold.testing.consumer import (
    CONFIG,
    file_contents,
    invoke,
    project,
    update_config,
    vcs_backend,
)
from tooling.worker.src.scaffold.testing.repository import (
    GATE,
    commit,
    feature_repository,
    hg,
    initialize_mercurial,
    resumed,
    revision,
)


@pytest.mark.parametrize("obstacle", ["none", "untracked", "hook", "invalid-name"])
def test_private_feature_start_preserves_native_state(
    worker: Path, tmp_path: Path, obstacle: str
) -> None:

    feature_repository(tmp_path, "hg")
    update_config(tmp_path / "agentrig.yaml", git={"backend": vcs_backend("private")})
    commit(tmp_path, "hg")
    base = revision(tmp_path, "hg")
    untracked = obstacle == "untracked"
    hook = obstacle == "hook"
    invalid = obstacle == "invalid-name"
    succeeds = obstacle == "none"
    if untracked:
        (tmp_path / "pending.txt").write_text("preserve me\n")
    if hook:
        (tmp_path / ".hg/hgrc").write_text("[hooks]\npre-branch.reject = false\n")
    name = "bad:name" if invalid else "with spaces"
    result = invoke(worker, tmp_path, "feature-start", name)
    assert result.returncode == (0 if succeeds else 2), result.stdout + result.stderr
    assert revision(tmp_path, "hg") == base
    branch = resumed(worker, tmp_path)["vcs"]["branch"]
    assert branch == ("task/with spaces" if succeeds else "default")
    assert (tmp_path / "src/value.py").read_text() == "value = 1\n"
    if untracked:
        assert (tmp_path / "pending.txt").read_text() == "preserve me\n"
    if hook:
        assert "pre-branch.reject" in result.stderr


def test_private_installed_commit_gate_preserves_failed_and_unselected_work(
    worker: Path, tmp_path: Path
) -> None:

    initialize_mercurial(worker, tmp_path)
    update_config(tmp_path / "agentrig.yaml", vcs={"backend": vcs_backend("private")})
    assert invoke(worker, tmp_path, "setup").returncode == 0
    (tmp_path / "src").mkdir()
    source = tmp_path / "src/test_sample.py"
    source.write_text("def test_value():\n    assert 1 == 1\n")
    unrelated = tmp_path / "src/other.py"
    unrelated.write_text("other = 1\n")
    hg(tmp_path, "add")
    hg(tmp_path, "branch", "task/bootstrap")
    hg(tmp_path, "commit", "-m", "bootstrap", "-u", "Test")
    base = revision(tmp_path, "hg")
    source.write_text("def test_value():\n    assert 1 == 2\n")
    unrelated.write_text("preserve unselected work\n")
    failed = selected_private_commit(tmp_path)
    assert failed.returncode != 0, failed.stdout + failed.stderr
    assert "FAILED" in failed.stdout
    assert revision(tmp_path, "hg") == base
    assert source.read_text().endswith("assert 1 == 2\n")
    source.write_text("def test_value():\n    assert 2 == 2\n")
    passed = selected_private_commit(tmp_path)
    assert passed.returncode == 0, passed.stdout + passed.stderr
    assert revision(tmp_path, "hg") != base
    assert unrelated.read_text() == "preserve unselected work\n"
    assert hg(tmp_path, "cat", "-r", ".", "src/other.py") == "other = 1"


def selected_private_commit(root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["hg", "commit", "-m", "candidate", "-u", "Test", "src/test_sample.py"],
        cwd=root,
        capture_output=True,
        text=True,
        timeout=30,
    )


@pytest.mark.parametrize(
    "case", [("task/candidate", False, 0), ("trunk", False, 1), ("trunk", True, 0)]
)
def test_private_guard_uses_exact_opaque_revision_without_native_metadata(
    worker: Path, tmp_path: Path, case: tuple[str, bool, int]
) -> None:
    project(tmp_path, CONFIG + GATE)
    (tmp_path / "src/value.py").write_text("value = 1\n")
    files = [str(path.relative_to(tmp_path)) for path in file_contents(tmp_path)]
    tree = [{"path": path, "kind": "file", "object": "blob-42"} for path in files]
    branch, merge, expected = case
    script = (
        "import json,sys,pathlib; r=json.load(sys.stdin); op=r['operation']; a=r['arguments']; "
        "assert op=='resolve' or a['revision']=='revision-42'; "
        f"values={{'resolve':'revision-42','tree':{tree!r},'commit-context':[{branch!r},{merge!r}]}}; "
        "result=list(pathlib.Path(a['path']).read_bytes()) if op=='read' else values[op]; "
        "print(json.dumps({'version':1,'result':result}))"
    )
    update_config(
        tmp_path / "agentrig.yaml", git={"backend": {"command": ["python3", "-c", script]}}
    )
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "guard-commit", "--revision", "moving-reference")
    assert result.returncode == expected, result.stdout + result.stderr
    assert file_contents(tmp_path) == before
    assert not (tmp_path / ".git").exists() and not (tmp_path / ".hg").exists()
