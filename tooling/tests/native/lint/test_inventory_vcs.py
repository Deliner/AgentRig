import json
import shlex
import subprocess
from pathlib import Path
from typing import Any

import pytest

from tooling.worker.src.lint.testing.consumer import CONFIG, prepare


def inventory_project(root: Path, vcs: str) -> None:
    source = root / "src"
    source.mkdir(parents=True)
    for name in ["tracked.py", "deleted.py"]:
        (source / name).write_text("line\n" * 6)
    subprocess.run([vcs, "init"], cwd=root, capture_output=True, check=True)
    subprocess.run([vcs, "add", "src"], cwd=root, capture_output=True, check=True)
    git = vcs == "git"
    args = (
        ["-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-qm", "base"]
        if git
        else ["commit", "-m", "base", "-u", "Test"]
    )
    subprocess.run([vcs, *args], cwd=root, capture_output=True, check=True)
    ignore = root / (".gitignore" if git else ".hgignore")
    ignore.write_text(
        "ignored.py\ntracked.py\n" if git else "syntax: glob\n**/ignored.py\n**/tracked.py\n"
    )
    for name in ["added.py", "unknown.py", "ignored.py"]:
        (source / name).write_text("line\n" * 6)
    subprocess.run([vcs, "add", "src/added.py"], cwd=root, capture_output=True, check=True)
    (source / "deleted.py").unlink()
    (source / "link.py").symlink_to("tracked.py")
    (root / (".git" if git else ".hg") / "control.py").write_text("line\n" * 6)


@pytest.mark.parametrize("vcs", ["git", "hg"])
def test_both_linters_use_native_tracked_and_untracked_inventory(
    worker: Path, tmp_path: Path, vcs: str
) -> None:
    policy = tmp_path / "policy"
    policy.mkdir()
    prepare(policy, 'skill_root: "."\n' + CONFIG.replace('include: ["src/**"]', 'include: ["**"]'))
    root = tmp_path / "consumer"
    inventory_project(root, vcs)
    before = {path.name: path.read_bytes() for path in (root / "src").iterdir()}
    findings = []
    for binary in [worker, worker.with_name("agentrig-lint")]:
        embedded = binary == worker
        prefix = ["lint"] if embedded else []
        result = subprocess.run(
            [
                str(binary),
                *prefix,
                "--root",
                str(root),
                "--config",
                str(policy / "lint.yaml"),
                "--json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        assert result.returncode == 1, result.stdout + result.stderr
        items = json.loads(result.stdout)
        assert {item["path"] for item in items} == {
            "src/tracked.py",
            "src/added.py",
            "src/unknown.py",
        }
        for item in items:
            item.pop("rerun")
        findings.append(items)
    assert findings[0] == findings[1]
    assert {path.name: path.read_bytes() for path in (root / "src").iterdir()} == before


def private_inventory(root: Path) -> tuple[Path, list[str]]:
    policy = root / "policy with spaces"
    policy.mkdir()
    prepare(policy, 'skill_root: "."\n' + CONFIG)
    consumer = root / "consumer"
    inventory_project(consumer, "hg")
    script = Path(__file__).resolve().parents[3] / "worker/examples/external_vcs.py"
    selected = policy / "vcs.yaml"
    selected.write_text(json.dumps({"command": ["python3", "-B", str(script)]}))
    return consumer, [
        "--root",
        str(consumer),
        "--config",
        str(policy / "lint.yaml"),
        "--vcs-config",
        str(selected),
    ]


@pytest.mark.parametrize("command", ["lint", "lint-config-check", "lint-explain"])
def test_private_inventory_is_shared_by_both_linter_clis(
    worker: Path, tmp_path: Path, command: str
) -> None:
    root, args = private_inventory(tmp_path)
    before = file_contents(root)
    explaining = command == "lint-explain"
    checking = command == "lint-config-check"
    inspection = explaining or checking
    selected = ["src/ignored.py"] if explaining else []
    results = []
    for binary in [worker, worker.with_name("agentrig-lint")]:
        result = subprocess.run(
            [str(binary), command, *selected, *args, "--json"],
            capture_output=True,
            text=True,
            check=False,
        )
        assert result.returncode == (0 if inspection else 1), result.stdout + result.stderr
        value = json.loads(result.stdout)
        if explaining:
            assert not value["rules"][0]["selected"]
            assert "absent from selected inventory" in value["rules"][0]["reason"]
        elif checking:
            assert value == []
        else:
            verify_findings(value)
        results.append(value)
    assert results[0] == results[1]
    assert file_contents(root) == before


def file_contents(root: Path) -> dict[Path, bytes]:
    return {path: path.read_bytes() for path in filter(Path.is_file, root.rglob("*"))}


def verify_findings(items: list[dict[str, Any]]) -> None:
    assert {item["path"] for item in items} == {"src/tracked.py", "src/added.py", "src/unknown.py"}
    for item in items:
        rerun = item.pop("rerun")
        assert "--vcs-config" in rerun
        repeated = subprocess.run(shlex.split(rerun), capture_output=True, text=True, check=False)
        assert repeated.returncode == 1, repeated.stdout + repeated.stderr
        assert "src/ignored.py" not in repeated.stdout


@pytest.mark.parametrize(
    "declaration",
    [
        {"command": []},
        {"command": ["python3"], "unknown": True},
        {"command": ["python3", "-c", "import sys; sys.exit(64)"]},
    ],
)
def test_private_linter_selection_never_falls_back_to_native_inventory(
    worker: Path, tmp_path: Path, declaration: dict[str, object]
) -> None:
    root, args = private_inventory(tmp_path)
    Path(args[-1]).write_text(json.dumps(declaration))
    for binary in [worker, worker.with_name("agentrig-lint")]:
        result = subprocess.run(
            [str(binary), "lint", *args, "--json"], capture_output=True, text=True, check=False
        )
        assert result.returncode == 2, result.stdout + result.stderr
        assert "src/tracked.py" not in result.stdout
        assert "external VCS" in result.stdout + result.stderr or "unknown field" in result.stderr
