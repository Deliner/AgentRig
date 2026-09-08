import shlex
import subprocess
from pathlib import Path

import pytest

from tooling.worker.src.scaffold.testing.consumer import (
    invoke,
    update_config,
    vcs_backend,
)
from tooling.worker.src.scaffold.testing.repository import commit, repository, resumed, revision


def merge_repository(worker: Path, root: Path, vcs: str = "hg") -> tuple[str, str]:
    repository(root, vcs)
    update_config(
        root / "agentrig.yaml",
        git={"backend": vcs_backend(vcs), "base": "default"},
        commands={
            "fail": {
                "argv": [
                    "sh",
                    "-c",
                    'test -z "$BLOCK_DELIVERY" && if [ -n "$MUTATE_MERGE" ]; then printf "value = 99\\n" > src/value.py; fi',
                ]
            }
        },
    )
    base = commit(root, "hg")
    (root / ".hg/hgrc").write_text(
        f"[ui]\nusername = Test\n[hooks]\npretxncommit.agentrig = {shlex.quote(str(worker))} "
        'check --root . --revision "$HG_NODE"\npretxncommit.reject = test -z "$BLOCK_COMMIT"\n'
    )
    assert invoke(worker, root, "feature-start", "product").returncode == 0
    (root / "src/value.py").write_text("value = 2\n")
    return base, commit(root, "hg")


@pytest.mark.parametrize(
    "scenario", [(vcs, phase) for vcs in ["hg", "private"] for phase in ["gate", "commit-hook"]]
)
def test_mercurial_integration_recovers_failed_gate(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, scenario: tuple[str, str]
) -> None:
    vcs, phase = scenario
    base, candidate = merge_repository(worker, tmp_path, vcs)
    during_gate = phase == "gate"
    variable = "BLOCK_DELIVERY" if during_gate else "BLOCK_COMMIT"
    monkeypatch.setenv(variable, "yes")
    failed = invoke(worker, tmp_path, "feature-merge")
    assert failed.returncode != 0, failed.stdout + failed.stderr
    assert revision(tmp_path, "hg") == base
    assert resumed(worker, tmp_path)["vcs"]["merge_in_progress"]
    monkeypatch.delenv(variable)
    result = invoke(worker, tmp_path, "feature-merge")
    assert result.returncode == 0, result.stdout + result.stderr
    parents = subprocess.check_output(
        ["hg", "log", "-r", ".", "-T", "{p1node} {p2node}"], cwd=tmp_path, text=True
    ).split()
    assert parents == [base, candidate]
    observed = resumed(worker, tmp_path)["vcs"]
    assert observed["branch"] == "default" and observed["status"] == ""
    assert not observed["merge_in_progress"]
    assert (tmp_path / "src/value.py").read_text() == "value = 2\n"


@pytest.mark.parametrize("vcs", ["hg", "private"])
def test_mercurial_integration_rejects_mutated_gate_inputs(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, vcs: str
) -> None:
    base, _ = merge_repository(worker, tmp_path, vcs)
    monkeypatch.setenv("MUTATE_MERGE", "yes")
    result = invoke(worker, tmp_path, "feature-merge")
    assert result.returncode == 2 and "checked merge inputs changed" in result.stderr
    assert revision(tmp_path, "hg") == base
    assert resumed(worker, tmp_path)["vcs"]["merge_in_progress"]
    assert (tmp_path / "src/value.py").read_text() == "value = 99\n"


@pytest.mark.parametrize("vcs", ["hg", "private"])
def test_mercurial_integration_recovers_native_conflict(
    worker: Path, tmp_path: Path, vcs: str
) -> None:
    base, candidate = merge_repository(worker, tmp_path, vcs)
    subprocess.run(["hg", "update", "-r", base], cwd=tmp_path, capture_output=True, check=True)
    source = tmp_path / "src/value.py"
    source.write_text("value = 3\n")
    base = commit(tmp_path, "hg")
    subprocess.run(["hg", "update", "-r", candidate], cwd=tmp_path, capture_output=True, check=True)
    result = invoke(worker, tmp_path, "feature-merge")
    assert result.returncode != 0 and "conflict" in result.stderr
    assert resumed(worker, tmp_path)["vcs"]["merge_in_progress"]
    source.write_text("value = 4\n")
    source.with_suffix(".py.orig").unlink(missing_ok=True)
    subprocess.run(
        ["hg", "resolve", "-m", str(source)], cwd=tmp_path, capture_output=True, check=True
    )
    result = invoke(worker, tmp_path, "feature-merge")
    assert result.returncode == 0, result.stdout + result.stderr
    assert source.read_text() == "value = 4\n"
    assert (
        subprocess.check_output(
            ["hg", "log", "-r", candidate, "-T", "{branch}"], cwd=tmp_path, text=True
        )
        == "task/product"
    )
