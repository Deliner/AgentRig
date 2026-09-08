import json
import os
import select
import subprocess
import time
from pathlib import Path

import pytest
import yaml

from tooling.worker.src.scaffold.package.setup.tests.consumer import delegated_project
from tooling.worker.src.scaffold.testing.consumer import (
    file_contents,
    invoke,
)


def interactive(worker: Path, root: Path, answers: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(worker), "init", "--interactive", "--root", str(root)],
        input="".join(answer + "\n" for answer in answers),
        capture_output=True,
        text=True,
        check=False,
    )


@pytest.mark.parametrize("language", ["python", "rust"])
@pytest.mark.parametrize("vcs", ["git", "mercurial"])
def test_interactive_init_matches_declarative_setup(
    worker: Path, tmp_path: Path, language: str, vcs: str
) -> None:
    target = tmp_path / "interactive"
    answers = [""] * 14
    answers[1] = language
    answers[9] = vcs
    answers[-1] = "yes"
    result = interactive(worker, target, answers)
    assert result.returncode == 0, result.stdout + result.stderr
    ordinary = tmp_path / "ordinary"
    ordinary.mkdir()
    assert invoke(worker, ordinary, "init", "--language", language, "--vcs", vcs).returncode == 0
    assert invoke(worker, ordinary, "setup").returncode == 0
    for name in [
        "agentrig.yaml",
        "AGENTS.md",
        "justfile",
        ".codex/config.toml",
        ".agentrig/manifest.json",
    ]:
        assert (target / name).read_bytes() == (ordinary / name).read_bytes(), name
    assert "Configuration for" in result.stdout
    assert '"preview": true' in result.stdout
    assert invoke(target / ".agentrig/bin/agentrig", target, "config-check").returncode == 0


@pytest.mark.parametrize("answers", [[], ["cancel"], [""] * 13, [""] * 13 + ["no"]])
def test_interactive_cancellation_leaves_no_target(
    worker: Path, tmp_path: Path, answers: list[str]
) -> None:
    target = tmp_path / "missing" / "consumer"
    result = interactive(worker, target, answers)
    assert result.returncode == 0, result.stdout + result.stderr
    assert "cancelled" in result.stdout
    assert not (tmp_path / "missing").exists()


def test_interactive_invalid_selection_preserves_existing_files(
    worker: Path, tmp_path: Path
) -> None:
    (tmp_path / "user.txt").write_text("Keep user content.\n")
    before = file_contents(tmp_path)
    answers = [""] * 14
    answers[1] = "unsupported-language"
    result = interactive(worker, tmp_path, answers)
    assert result.returncode == 2
    assert "python or rust" in result.stderr
    assert file_contents(tmp_path) == before


def test_interactive_selects_layout_checks_review_and_delegation(
    worker: Path, tmp_path: Path
) -> None:
    root = delegated_project(worker, tmp_path)
    (root / "agentrig.yaml").unlink()
    answers = [""] * 14
    answers[2] = "wizard rig"
    answers[4] = "notes"
    answers[8] = "true"
    answers[10] = "claude-code"
    answers[11] = "agents/profiles.yaml"
    answers[12] = "lint,memory"
    answers[13] = "yes"
    result = interactive(worker, root, answers)
    assert result.returncode == 0, result.stdout + result.stderr
    config = yaml.safe_load((root / "agentrig.yaml").read_text())
    assert config["paths"]["service"] == "wizard rig"
    assert config["paths"]["skills"] == "wizard rig/skills"
    assert config["paths"]["memory"] == "notes"
    assert {check["id"] for check in config["checks"]} == {"lint", "memory"}
    settings = json.loads((root / ".mcp.json").read_text())
    assert set(settings["mcpServers"]) == {"worker_review", "worker_delegation"}
    installed = root / "wizard rig/bin/agentrig"
    assert invoke(installed, root, "delegate", "config-check").returncode == 0
    assert invoke(installed, root, "review", "config-check").returncode == 0


def wait_for_preview(process: subprocess.Popen[bytes]) -> None:
    assert process.stdout is not None
    output = b""
    deadline = time.monotonic() + 10
    while b'"preview": true' not in output:
        ready, _, _ = select.select([process.stdout], [], [], max(0, deadline - time.monotonic()))
        assert ready, output.decode(errors="replace")
        chunk = os.read(process.stdout.fileno(), 65536)
        assert chunk, output.decode(errors="replace")
        output += chunk


def test_interactive_preserves_changes_after_preview(worker: Path, tmp_path: Path) -> None:
    with subprocess.Popen(
        [str(worker), "init", "--interactive", "--root", str(tmp_path)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    ) as process:
        try:
            assert process.stdin is not None
            process.stdin.write(b"\n" * 13)
            process.stdin.flush()
            wait_for_preview(process)
            note = tmp_path / "AGENTS.md"
            note.write_text("User change after preview.\n")
            output, errors = process.communicate(b"yes\n", timeout=10)
            assert process.returncode == 2, output + errors
            assert b"setup input changed" in errors
            assert note.read_text() == "User change after preview.\n"
            assert not (tmp_path / ".git").exists()
            assert not (tmp_path / "agentrig.yaml").exists()
        finally:
            alive = process.poll() is None
            if alive:
                process.kill()
            process.wait(timeout=10)


@pytest.mark.parametrize("wizard", [True, False])
def test_init_requires_explicit_legacy_migration(
    worker: Path, tmp_path: Path, wizard: bool
) -> None:
    legacy = tmp_path / "worker.toml"
    legacy.write_text("# Preserve the legacy declaration for explicit migration.\n")
    before = file_contents(tmp_path)
    if wizard:
        result = interactive(worker, tmp_path, [""] * 13 + ["yes"])
    else:
        result = invoke(worker, tmp_path, "init")
    assert result.returncode == 2
    assert "explicit upgrade" in result.stderr
    assert file_contents(tmp_path) == before
