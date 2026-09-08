import json
import subprocess
from pathlib import Path

import pytest

from tooling.worker.src.lint.architecture.tests.test_architecture import contract, snapshot
from tooling.worker.src.lint.architecture.tests.test_architecture_inventory import documents


def repository(root: Path, backend: str) -> list[str]:
    private = backend == "external"
    git = backend == "git"
    native = "git" if git else "hg"
    subprocess.run([native, "init"], cwd=root, capture_output=True, check=True)
    ignore = root / (".gitignore" if git else ".hgignore")
    ignore.write_text("ignored/\n" if git else "syntax: glob\nignored\n")
    if private:
        adapter = Path(__file__).resolve().parents[4] / "examples/external_vcs.py"
        selection = root / "vcs.yaml"
        selection.write_text(json.dumps({"command": ["python3", "-B", str(adapter)]}))
        return ["--vcs-config", str(selection)]
    return []


@pytest.mark.parametrize("backend", ["git", "hg", "external"])
@pytest.mark.parametrize("standalone", [False, True])
def test_empty_vcs_directories_require_contracts_without_including_ignored_trees(
    worker: Path, tmp_path: Path, backend: str, standalone: bool
) -> None:
    docs = documents(tmp_path)
    empty = docs / "drafts"
    empty.mkdir()
    contract(docs)
    args = repository(tmp_path, backend)
    (docs / "ignored/nested").mkdir(parents=True)
    (docs / "generated/nested").mkdir(parents=True)
    policy = tmp_path / "lint.yaml"
    policy.write_text(
        "exclude: ['src/docs/generated', 'src/docs/generated/**']\n" + policy.read_text()
    )
    binary = worker.with_name("agentrig-lint") if standalone else worker
    command = [str(binary), "lint", "--root", str(tmp_path), "--json", *args]
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


@pytest.mark.parametrize("standalone", [False, True])
def test_private_directory_inventory_cannot_silently_fall_back(
    worker: Path, tmp_path: Path, standalone: bool
) -> None:
    documents(tmp_path)
    script = (
        "import json,sys; request=json.load(sys.stdin); "
        "operation=request['operation']; "
        "sys.exit(64) if operation == 'working-directories' else None; "
        "print(json.dumps({'version':1,'result':[]}))"
    )
    selection = tmp_path / "vcs.yaml"
    selection.write_text(json.dumps({"command": ["python3", "-c", script]}))
    binary = worker.with_name("agentrig-lint") if standalone else worker
    result = subprocess.run(
        [str(binary), "lint", "--root", str(tmp_path), "--vcs-config", str(selection), "--json"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 2, result.stdout + result.stderr
    assert "external VCS does not support working-directories" in result.stdout


@pytest.mark.parametrize("backend", ["git", "hg", "external"])
def test_empty_directories_preserve_file_derived_size_measurement(
    worker: Path, tmp_path: Path, backend: str
) -> None:
    docs = documents(tmp_path)
    args = repository(tmp_path, backend)
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
        [str(worker), "lint", "--root", str(tmp_path), "--json", *args],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 1, result.stdout + result.stderr
    assert [item["actual"] for item in json.loads(result.stdout)] == [2]
