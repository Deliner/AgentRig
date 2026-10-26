import os
import select
import subprocess
import time
from pathlib import Path

import pytest
import yaml

from tooling.worker.src.scaffold.testing.consumer import (
    file_contents,
    invoke,
)

QUESTIONS = 11


def interactive(worker: Path, root: Path, answers: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(worker), "init", "--interactive", "--root", str(root)],
        input="".join(answer + "\n" for answer in answers),
        capture_output=True,
        text=True,
        check=False,
    )


@pytest.mark.parametrize("language", ["python", "rust"])
def test_interactive_init_matches_declarative_init(
    worker: Path, tmp_path: Path, language: str
) -> None:
    target = tmp_path / "interactive"
    answers = [""] * QUESTIONS
    answers[1] = language
    answers[-1] = "yes"
    result = interactive(worker, target, answers)
    assert result.returncode == 0, result.stdout + result.stderr
    ordinary = tmp_path / "ordinary"
    ordinary.mkdir()
    assert invoke(worker, ordinary, "init", "--language", language).returncode == 0
    for name in ["agentrig.yaml", "AGENTS.md", "justfile", ".codex/config.toml"]:
        assert (target / name).read_bytes() == (ordinary / name).read_bytes(), name
    assert "Configuration for" in result.stdout
    assert invoke(target / ".agentrig/bin/agentrig", target, "config-check").returncode == 0


@pytest.mark.parametrize(
    "answers", [[], ["cancel"], [""] * (QUESTIONS - 1), [""] * (QUESTIONS - 1) + ["no"]]
)
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
    answers = [""] * QUESTIONS
    answers[1] = "unsupported-language"
    result = interactive(worker, tmp_path, answers)
    assert result.returncode == 2
    assert "python or rust" in result.stderr
    assert file_contents(tmp_path) == before


def test_interactive_selects_layout_client_and_checks(worker: Path, tmp_path: Path) -> None:
    answers = [""] * QUESTIONS
    answers[2] = "wizard rig"
    answers[4] = "notes"
    answers[8] = "claude-code"
    answers[9] = "lint,memory"
    answers[10] = "yes"
    result = interactive(worker, tmp_path, answers)
    assert result.returncode == 0, result.stdout + result.stderr
    config = yaml.safe_load((tmp_path / "agentrig.yaml").read_text())
    assert config["paths"]["service"] == "wizard rig"
    assert config["paths"]["skills"] == "wizard rig/skills"
    assert config["paths"]["memory"] == "notes"
    assert {check["id"] for check in config["checks"]} == {"lint", "memory"}
    assert (tmp_path / ".claude/settings.json").is_file()
    assert not (tmp_path / ".mcp.json").exists()
    installed = tmp_path / "wizard rig/bin/agentrig"
    assert invoke(installed, tmp_path, "config-check").returncode == 0


def wait_for_files(process: subprocess.Popen[bytes]) -> None:
    assert process.stdout is not None
    output = b""
    deadline = time.monotonic() + 10
    while b"Install this environment?" not in output:
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
            process.stdin.write(b"\n" * (QUESTIONS - 1))
            process.stdin.flush()
            wait_for_files(process)
            note = tmp_path / "AGENTS.md"
            note.write_text("User change after preview.\n")
            output, errors = process.communicate(b"yes\n", timeout=10)
            assert process.returncode == 2, output + errors
            assert b"collision" in errors
            assert note.read_text() == "User change after preview.\n"
            assert not (tmp_path / ".git").exists()
            assert not (tmp_path / "agentrig.yaml").exists()
        finally:
            alive = process.poll() is None
            if alive:
                process.kill()
            process.wait(timeout=10)
