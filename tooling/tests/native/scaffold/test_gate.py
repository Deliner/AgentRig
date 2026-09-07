import json
import shlex
import shutil
import subprocess
from pathlib import Path

import pytest
from support import CONFIG, file_contents, invoke, project, update_config, vcs_backend

GATE = """
checks:
- id: "lint"
  kind: "lint"
  skill: "guides/repair/SKILL.md"
- id: "tests"
  kind: "command"
  command: "fail"
  skill: "guides/repair/SKILL.md"
  warning: true
"""


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
        config = tmp_path / "agentrig.yaml"
        config.write_text(config.read_text().replace('command: "fail"', 'command: "unknown"'))
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


def test_private_project_selection_rejects_native_fallback(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, CONFIG + GATE)
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    config = tmp_path / "agentrig.yaml"
    adapter = {"command": ["python3", "-c", "import sys; sys.exit(64)"]}
    update_config(config, git={"backend": adapter})
    assert invoke(worker, tmp_path, "config-check").returncode == 0
    for args in [("check",), ("check", "--revision", "tip"), ("feature-start", "example")]:
        result = invoke(worker, tmp_path, *args)
        assert result.returncode == 2, result.stdout + result.stderr
        assert "external VCS" in result.stderr or "private VCS" in result.stderr
    update_config(config, git={"backend": {"command": []}})
    assert invoke(worker, tmp_path, "config-check").returncode == 2


def test_private_worktree_inventory_controls_lint_and_check_selection(
    worker: Path, tmp_path: Path
) -> None:
    project(tmp_path, CONFIG + GATE)
    (tmp_path / "src/large.py").write_text("line\n" * 61)
    files = ["agentrig.yaml", "lint.yaml", "guides/repair/SKILL.md"]
    replies = {"head": "r1", "working-files": files}
    script = f"import json,sys; r=json.load(sys.stdin); print(json.dumps({{'version':1,'result':{replies!r}[r['operation']]}}))"
    update_config(
        tmp_path / "agentrig.yaml", git={"backend": {"command": ["python3", "-c", script]}}
    )
    result = invoke(worker, tmp_path, "check", "--only", "lint")
    assert result.returncode == 0, result.stdout + result.stderr
    assert "exceeds 60" not in result.stdout
    assert json.loads((tmp_path / ".runtime/checks.json").read_text())["revision"] == "r1"


def test_private_resume_accepts_opaque_recorded_revisions(worker: Path, tmp_path: Path) -> None:
    from test_memory import memory

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


@pytest.mark.parametrize("obstacle", ["none", "untracked", "hook", "invalid-name"])
def test_private_feature_start_preserves_native_state(
    worker: Path, tmp_path: Path, obstacle: str
) -> None:
    from test_feedback import commit, resumed, revision
    from test_git import feature_repository

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


def test_private_setup_preview_resolves_adapter_from_consumer_root(
    worker: Path, tmp_path: Path
) -> None:
    from test_setup import declaration

    root = declaration(worker, tmp_path, "private rig")
    backend = vcs_backend("private")
    assert isinstance(backend, dict)
    shutil.copy2(backend["command"][-1], root / "adapter.py")
    backend["command"][-1] = "adapter.py"
    update_config(root / "agentrig.yaml", vcs={"backend": backend})
    before = file_contents(root)
    result = subprocess.run(
        [str(worker), "setup", "--root", str(root), "--preview"],
        cwd=tmp_path,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr
    preview = json.loads(result.stdout)
    assert preview["registrations"]["vcs"]["initialize"] is True
    assert file_contents(root) == before
    assert not (root / ".hg").exists()
    applied = invoke(worker, root, "setup")
    assert applied.returncode == 0, applied.stdout + applied.stderr
    installed = root / "private rig/bin/agentrig"
    assert invoke(installed, root, "doctor").returncode == 0
    repeated = invoke(installed, root, "setup", "--preview")
    assert repeated.returncode == 0, repeated.stderr
    assert json.loads(repeated.stdout)["files"] == []
    assert json.loads(repeated.stdout)["registrations"]["vcs"]["initialize"] is False


def test_private_installed_commit_gate_preserves_failed_and_unselected_work(
    worker: Path, tmp_path: Path
) -> None:
    from test_feedback import revision
    from test_git import hg, initialize_mercurial

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
