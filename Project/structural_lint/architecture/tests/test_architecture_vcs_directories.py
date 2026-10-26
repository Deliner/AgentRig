import json
import subprocess
from pathlib import Path

import pytest

from Project.structural_lint.architecture.tests.test_architecture import contract, snapshot
from Project.structural_lint.architecture.tests.test_architecture_inventory import documents


def repository(root: Path) -> None:
    subprocess.run(["git", "init"], cwd=root, capture_output=True, check=True)
    (root / ".gitignore").write_text("ignored/\n")


@pytest.mark.parametrize("standalone", [False, True])
def test_empty_vcs_directories_require_contracts_without_including_ignored_trees(
    worker: Path, tmp_path: Path, standalone: bool
) -> None:
    docs = documents(tmp_path)
    empty = docs / "drafts"
    empty.mkdir()
    contract(docs)
    repository(tmp_path)
    (docs / "ignored/nested").mkdir(parents=True)
    (docs / "generated/nested").mkdir(parents=True)
    policy = tmp_path / "lint.yaml"
    policy.write_text(
        "exclude: ['src/docs/generated', 'src/docs/generated/**']\n" + policy.read_text()
    )
    binary = worker.with_name("agentrig-lint") if standalone else worker
    command = [str(binary), "lint", "--root", str(tmp_path), "--json"]
    before = snapshot(tmp_path)
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    findings = json.loads(result.stdout)
    assert result.returncode == 1, result.stdout + result.stderr
    assert [item["path"] for item in findings] == ["src/docs/drafts/architecture.yaml"]
    assert "missing directory contract" in findings[0]["message"]
    assert snapshot(tmp_path) == before
    contract(empty)
    result = subprocess.run(command, capture_output=True, text=True, check=False)
    assert result.returncode == 0, result.stdout + result.stderr
    assert json.loads(result.stdout) == []


def test_empty_directories_preserve_file_derived_size_measurement(
    worker: Path, tmp_path: Path
) -> None:
    docs = documents(tmp_path)
    repository(tmp_path)
    (docs / "drafts").mkdir()
    rule = {
        "id": "size",
        "kind": "directory-entries",
        "target": "directory",
        "include": ["src/docs"],
        "warning": 0,
        "error": 1,
        "warning_skill": ".agents/skills/repair/SKILL.md",
        "error_skill": ".agents/skills/repair/SKILL.md",
    }
    (tmp_path / "lint.yaml").write_text(
        json.dumps({"version": 1, "config_skill": rule["error_skill"], "rules": [rule]})
    )
    result = subprocess.run(
        [str(worker), "lint", "--root", str(tmp_path), "--json"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 1, result.stdout + result.stderr
    assert [item["actual"] for item in json.loads(result.stdout)] == [2]
