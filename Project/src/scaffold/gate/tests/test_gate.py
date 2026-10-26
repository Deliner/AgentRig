import json
import shlex
import shutil
import subprocess
from collections.abc import Callable
from pathlib import Path

import pytest
import yaml

from Project.src.scaffold.testing.consumer import (
    CONFIG,
    git,
    invoke,
    project,
    update_config,
)
from Project.src.scaffold.testing.repository import GATE


def affected_project(root: Path) -> None:
    project(root)
    groups = [
        {"include": ["src/a.rs"], "targets": ["tests/a", "tests/shared"]},
        {"include": ["src/b.rs", "src/moved.rs"], "targets": ["tests/b", "tests/shared"]},
        {"include": ["docs/**"], "targets": []},
    ]
    update_config(
        root,
        commands={
            "test": {
                "argv": [
                    "python3",
                    "-c",
                    "import json,sys; print('TARGETS='+json.dumps(sys.argv[1:]))",
                ],
                "accepts_args": True,
            }
        },
        checks=[
            {
                "id": "tests",
                "kind": "command",
                "command": "test",
                "skill": "guides/repair/SKILL.md",
                "affected": groups,
            }
        ],
    )
    affected_baseline(root)


def affected_baseline(root: Path) -> None:
    (root / ".gitignore").write_text(".runtime/\n")
    (root / "src/a.rs").write_text("before\n")
    (root / "src/b.rs").write_text("before\n")
    (root / "docs").mkdir()
    (root / "docs/guide.md").write_text("before\n")
    git(root, "init", "-b", "task/selection")
    git(root, "add", ".")
    git(root, "config", "user.name", "Test")
    git(root, "config", "user.email", "test@example.invalid")
    git(root, "commit", "-qm", "base")


def affected_check(worker: Path, root: Path, case: str) -> subprocess.CompletedProcess[str]:
    merge = case == "merge"
    if merge:
        git(root, "branch", "trunk")
        git(root, "commit", "-qm", "feature")
        return invoke(worker, root, "feature-merge")
    full = case == "full"
    args = [] if full else ["--staged"]
    return invoke(worker, root, "check", *args)


def affected_change(tmp_path: Path, case: str) -> None:
    source = tmp_path / "src/a.rs"

    def edit() -> int:
        return source.write_text("changed\n")

    actions: dict[str, Callable[[], object]] = {
        "edit": edit,
        "unstaged": edit,
        "full": edit,
        "merge": edit,
        "delete": source.unlink,
        "rename": lambda: source.rename(tmp_path / "src/moved.rs"),
        "unknown": lambda: (tmp_path / "unknown.txt").write_text("new\n"),
        "docs": lambda: (tmp_path / "docs/guide.md").write_text("changed\n"),
    }
    actions[case]()
    git(tmp_path, "add", "-A")
    unstaged = case == "unstaged"
    if unstaged:
        (tmp_path / "src/b.rs").write_text("not staged\n")


@pytest.mark.parametrize(
    "case", ["edit", "delete", "rename", "unknown", "docs", "unstaged", "full", "merge"]
)
def test_affected_groups_select_commit_and_merge_targets(
    worker: Path, tmp_path: Path, case: str
) -> None:
    affected_project(tmp_path)
    affected_change(tmp_path, case)
    result = affected_check(worker, tmp_path, case)
    assert result.returncode == 0, result.stdout + result.stderr
    expected = {
        "rename": ["tests/a", "tests/b", "tests/shared"],
        "unknown": [],
        "full": [],
    }
    docs = case == "docs"
    if docs:
        assert "TARGETS=" not in result.stdout
    else:
        assert (
            "TARGETS=" + json.dumps(expected.get(case, ["tests/a", "tests/shared"]))
            in result.stdout
        )
    record = json.loads((tmp_path / ".runtime/checks.json").read_text())
    assert record["selective"] == (case not in {"unknown", "full"})
    report = invoke(worker, tmp_path, "report")
    assert report.returncode == 0, report.stderr
    evidence = json.loads(report.stdout.split("Check evidence: ", 1)[1].splitlines()[0])
    assert evidence["full_gate_passed"] == (case in {"unknown", "full"})


@pytest.mark.parametrize(
    "include,targets", [([], []), (["["], []), (["src/**"], [""]), (["src/**"], ["a\0b"])]
)
def test_affected_groups_reject_invalid_configuration(
    worker: Path, tmp_path: Path, include: list[str], targets: list[str]
) -> None:
    affected_project(tmp_path)
    path = tmp_path / ".agentrig/checks.yaml"
    checks = yaml.safe_load(path.read_text())
    checks[0]["affected"] = [{"include": include, "targets": targets}]
    path.write_text(yaml.safe_dump(checks))
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2, result.stdout + result.stderr
    assert "affected" in result.stderr


def test_gate_warning_and_block(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, CONFIG + GATE)
    result = invoke(worker, tmp_path, "check")
    assert result.returncode == 0, result.stderr
    assert "WARNING [tests]" in result.stderr
    assert "guides/repair/SKILL.md" in result.stderr
    (tmp_path / "src/large.py").write_text("line\n" * 61)
    result = invoke(worker, tmp_path, "check")
    assert result.returncode == 1
    assert "ERROR [lint]" in result.stderr
    assert "WARNING [tests]" not in result.stderr


def test_staged_gate_uses_staged_configuration(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, CONFIG + GATE)
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    (tmp_path / "src/large.py").write_text("line\n" * 61)
    subprocess.run(["git", "add", "."], cwd=tmp_path, check=True)
    (tmp_path / "src/large.py").write_text("line\n")
    (tmp_path / "lint.yaml").write_text(
        (tmp_path / "lint.yaml").read_text().replace("error: 60", "error: 100")
    )
    assert invoke(worker, tmp_path, "check").returncode == 0
    result = invoke(worker, tmp_path, "check", "--staged")
    assert result.returncode == 1, result.stderr
    assert "exceeds 60" in result.stdout


def test_staged_git_check_preserves_invalid_unstaged_configuration(
    worker: Path, tmp_path: Path
) -> None:
    project(tmp_path, CONFIG + GATE)
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    subprocess.run(["git", "add", "."], cwd=tmp_path, check=True)
    config = tmp_path / ".agentrig/project.yaml"
    config.write_text("invalid: [\n")
    result = invoke(worker, tmp_path, "check", "--staged")
    assert result.returncode == 0, result.stdout + result.stderr
    assert config.read_text() == "invalid: [\n"


def test_missing_stage_command_is_actionable(worker: Path, tmp_path: Path) -> None:
    project(
        tmp_path,
        (CONFIG + GATE).replace(
            'argv: ["sh", "-c", "exit 23"]', 'argv: ["missing-worker-test-executable"]'
        ),
    )
    result = invoke(worker, tmp_path, "check")
    assert result.returncode == 0
    assert "cannot execute" in result.stderr
    assert "guides/repair/SKILL.md" in result.stderr


def test_unknown_stage_command_rejected(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, (CONFIG + GATE).replace('command: "fail"', 'command: "unknown"'))
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "checks.tests: unknown command unknown" in result.stderr


def test_targeted_rerun_keeps_warning_policy(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, CONFIG + GATE)
    result = invoke(worker, tmp_path, "check", "--only", "tests")
    assert result.returncode == 0, result.stderr
    assert "PASS [lint]" not in result.stdout
    assert "WARNING [tests]" in result.stderr
    assert "**: exited 23" in result.stderr
    command = result.stderr.split("RERUN: ", 1)[1].splitlines()[0]
    repeated = subprocess.run(shlex.split(command), capture_output=True, text=True, check=False)
    assert repeated.returncode == result.returncode
    assert repeated.stderr == result.stderr


@pytest.mark.parametrize(
    "args",
    [
        ("--only",),
        ("--only", "unknown"),
        ("--staged", "--staged"),
        ("--only", "lint", "--only", "tests"),
    ],
)
def test_invalid_check_selection(worker: Path, tmp_path: Path, args: tuple[str, ...]) -> None:
    project(tmp_path, CONFIG + GATE)
    result = invoke(worker, tmp_path, "check", *args)
    assert result.returncode == 2
    assert "PASS [" not in result.stdout


def test_staged_diagnostic_rerun_survives_export(worker: Path, tmp_path: Path) -> None:
    root = tmp_path / "project with spaces"
    project(root, CONFIG + GATE)
    (root / "src/large.py").write_text("line\n" * 61)
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    subprocess.run(["git", "add", "."], cwd=root, check=True)
    (root / "src/large.py").write_text("line\n")
    result = invoke(worker, root, "check", "--staged")
    assert result.returncode == 1
    for output in (result.stdout, result.stderr):
        command = output.split("RERUN: ", 1)[1].splitlines()[0]
        repeated = subprocess.run(shlex.split(command), capture_output=True, text=True, check=False)
        assert repeated.returncode == 1, repeated.stderr
        assert "exceeds 60" in repeated.stdout


@pytest.mark.parametrize("command", ["memory-check", "config-check", "run"])
def test_standalone_failure_has_executable_guidance(
    worker: Path, tmp_path: Path, command: str
) -> None:
    project(tmp_path, CONFIG + GATE)
    config_check = command == "config-check"
    catalog_command = command == "run"
    if config_check:
        config = tmp_path / ".agentrig/checks.yaml"
        config.write_text(config.read_text().replace("command: fail", "command: unknown"))
    args = ("run", "fail") if catalog_command else (command,)
    result = subprocess.run(
        [str(worker), *args], cwd=tmp_path, capture_output=True, text=True, check=False
    )
    assert result.returncode != 0
    assert "guides/repair/SKILL.md" in result.stderr
    command_line = result.stderr.split("RERUN: ", 1)[1].splitlines()[0]
    repeated = subprocess.run(
        shlex.split(command_line), capture_output=True, text=True, check=False
    )
    assert repeated.returncode == result.returncode, repeated.stderr


def test_retry_survives_running_binary_replacement(worker: Path, tmp_path: Path) -> None:
    config = (CONFIG + GATE).replace(
        "exit 23", "cp worker worker.next; mv worker.next worker; exit 23"
    )
    project(tmp_path, config)
    executable = tmp_path / "worker"
    shutil.copy2(worker, executable)
    result = invoke(executable, tmp_path, "run", "fail")
    assert result.returncode == 23, result.stderr
    command = result.stderr.split("RERUN: ", 1)[1].splitlines()[0]
    argv = shlex.split(command)
    assert Path(argv[0]).is_file(), command
    repeated = subprocess.run(argv, capture_output=True, text=True, check=False)
    assert repeated.returncode == 23, repeated.stderr
