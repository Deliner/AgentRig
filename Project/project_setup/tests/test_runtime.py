import os
import subprocess
from pathlib import Path

import pytest

from Project.shared.testing.consumer import git, invoke


def initialized(worker: Path, root: Path) -> Path:
    git(root, "init", "-q", "-b", "main")
    git(root, "config", "user.name", "Test")
    git(root, "config", "user.email", "test@example.invalid")
    assert invoke(worker, root, "init").returncode == 0
    (runtime,) = (root / ".agentrig/bin").glob("*/agentrig")
    return runtime.parent


def reporting(host: Path, system: str) -> dict[str, str]:
    host.mkdir()
    uname = host / "uname"
    uname.write_text(f"#!/bin/sh\necho {system}\n")
    uname.chmod(0o755)
    return {**os.environ, "PATH": f"{host}{os.pathsep}{os.environ['PATH']}"}


def pre_commit(root: Path, env: dict[str, str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(root / ".agentrig/hooks/pre-commit")],
        cwd=root,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )


# DECISION: D150
@pytest.mark.parametrize(
    ("system", "platform"), [("Darwin arm64", "macos-arm64"), ("Linux x86_64", "linux-x86_64")]
)
def test_hooks_run_the_runtime_built_for_the_reported_platform(
    worker: Path, tmp_path: Path, system: str, platform: str
) -> None:
    project = tmp_path / "project"
    project.mkdir()
    runtime = initialized(worker, project)
    runtime.rename(runtime.parent / platform)
    result = pre_commit(project, reporting(tmp_path / "host", system))
    assert result.returncode == 1, result.stderr
    assert "direct commits on main are prohibited" in result.stderr


# DECISION: D150
def test_hooks_without_a_runtime_for_the_platform_name_it_and_the_add_command(
    worker: Path, tmp_path: Path
) -> None:
    project = tmp_path / "project"
    project.mkdir()
    initialized(worker, project)
    result = pre_commit(project, reporting(tmp_path / "host", "Linux riscv64"))
    assert result.returncode == 2
    assert "no runtime for linux-riscv64" in result.stderr
    assert "agentrig runtime-add" in result.stderr


# DECISION: D150
def test_runtime_add_moves_a_single_binary_project_to_platform_runtimes(
    worker: Path, tmp_path: Path
) -> None:
    runtime = initialized(worker, tmp_path)
    launcher = tmp_path / ".agentrig/bin/agentrig"
    (runtime / "agentrig").replace(launcher)
    runtime.rmdir()
    git(tmp_path, "add", ".")
    git(tmp_path, "-c", "core.hooksPath=/dev/null", "commit", "-qm", "single binary")
    git(tmp_path, "switch", "-c", "feature/runtime")
    added = invoke(worker, tmp_path, "runtime-add")
    assert added.returncode == 0, added.stderr
    assert launcher.read_text(encoding="utf-8").startswith("#!/bin/sh\n")
    assert (runtime / "agentrig").read_bytes() == worker.read_bytes()
    git(tmp_path, "add", ".")
    git(tmp_path, "commit", "-qm", "platform runtimes")
    merged = invoke(launcher, tmp_path, "feature-merge")
    assert merged.returncode == 0, merged.stdout + merged.stderr
    assert git(tmp_path, "branch", "--show-current").stdout.strip() == "main"
    assert invoke(launcher, tmp_path, "doctor").returncode == 0
