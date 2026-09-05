import json
import subprocess
import tomllib
from pathlib import Path

import pytest
from support import file_contents, invoke


def declaration(worker: Path, root: Path) -> Path:
    seed = root / "seed"
    seed.mkdir()
    result = invoke(worker, seed, "init", "--review", "true")
    assert result.returncode == 0, result.stderr
    consumer = root / "consumer"
    consumer.mkdir()
    (consumer / "worker.toml").write_text(
        "# Consumer policy\n" + (seed / "worker.toml").read_text()
    )
    (consumer / "src").mkdir()
    (consumer / "src/value.py").write_text("value = 1\n")
    return consumer


def test_setup_prepares_and_repeats_without_losing_settings(worker: Path, tmp_path: Path) -> None:
    root = declaration(worker, tmp_path)
    (root / ".codex").mkdir()
    settings = "# Keep this comment\nmodel = 'consumer-model'\n[features]\nhooks = true # enabled\n"
    (root / ".codex/config.toml").write_text(settings)
    (root / ".codex/config.toml").chmod(0o600)
    result = invoke(worker, root, "setup")
    assert result.returncode == 0, result.stdout + result.stderr
    actual = (root / ".codex/config.toml").read_text()
    assert actual.startswith(settings)
    assert (root / ".codex/config.toml").stat().st_mode & 0o777 == 0o600
    assert (root / "worker.toml").read_text().startswith("# Consumer policy\n")
    config = tomllib.loads(actual)["mcp_servers"]["worker_review"]
    messages = [
        dict(jsonrpc="2.0", id=1, method="initialize", params=dict(protocolVersion="2025-11-25")),
        dict(jsonrpc="2.0", id=2, method="tools/list"),
    ]
    connected = subprocess.run(
        [config["command"], *config["args"]],
        cwd=root,
        input="".join(json.dumps(message) + "\n" for message in messages),
        capture_output=True,
        text=True,
        check=False,
    )
    assert connected.returncode == 0, connected.stderr
    assert len(json.loads(connected.stdout.splitlines()[1])["result"]["tools"]) == 2
    before = file_contents(root)
    result = invoke(worker, root, "setup")
    assert result.returncode == 0, result.stdout + result.stderr
    assert file_contents(root) == before


def test_setup_preserves_changed_review_and_memory(worker: Path, tmp_path: Path) -> None:
    root = declaration(worker, tmp_path)
    assert invoke(worker, root, "setup").returncode == 0
    review = root / ".worker/review/config/review.toml"
    review.write_text(
        review.read_text().replace('model = "gpt-5.6-luna"', 'model = "consumer-model"')
    )
    state = root / "memory/State.md"
    state.write_text(state.read_text().replace("Not recorded.", "Consumer-owned state."))
    before = file_contents(root)
    result = invoke(worker, root, "setup")
    assert result.returncode == 0, result.stdout + result.stderr
    assert file_contents(root) == before


def test_setup_rejects_conflicting_assets_without_writing(worker: Path, tmp_path: Path) -> None:
    root = declaration(worker, tmp_path)
    assert invoke(worker, root, "setup").returncode == 0
    skill = root / ".worker/skills/repair/SKILL.md"
    skill.write_text(skill.read_text() + "\nConsumer instruction.\n")
    before = file_contents(root)
    result = invoke(worker, root, "setup")
    assert result.returncode == 2
    assert "setup conflict" in result.stderr
    assert file_contents(root) == before


@pytest.mark.parametrize("settings", ["features=false\n", "[features]\nhooks=false\n"])
def test_setup_rejects_hook_conflicts_before_writing(
    worker: Path, tmp_path: Path, settings: str
) -> None:
    root = declaration(worker, tmp_path)
    (root / ".codex").mkdir()
    (root / ".codex/config.toml").write_text(settings)
    before = file_contents(root)
    result = invoke(worker, root, "setup")
    assert result.returncode == 2
    assert "setup conflict" in result.stderr
    assert file_contents(root) == before


def test_setup_rejects_unknown_capability_before_writing(worker: Path, tmp_path: Path) -> None:
    root = declaration(worker, tmp_path)
    config = root / "worker.toml"
    config.write_text(
        config.read_text().replace("[capabilities]", "[capabilities]\nunknown = true")
    )
    before = file_contents(root)
    result = invoke(worker, root, "setup")
    assert result.returncode == 2
    assert "unknown field" in result.stderr
    assert file_contents(root) == before
