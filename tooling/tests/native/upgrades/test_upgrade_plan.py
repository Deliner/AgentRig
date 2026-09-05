import hashlib
import json
import subprocess
from pathlib import Path
from typing import Any


def invoke(worker: Path, root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(worker), *args, "--root", str(root)], capture_output=True, text=True, check=False
    )


def prepare(worker: Path, predecessor: Path, root: Path) -> tuple[Path, dict[str, Any]]:
    installed = invoke(predecessor, root, "init", "--skills", "guides", "--memory", "notes")
    assert installed.returncode == 0, installed.stderr
    config = root / "worker.toml"
    config.write_text(config.read_text() + "\n# preserve this project comment\n")
    skill = root / "guides/repair/SKILL.md"
    skill.write_text(skill.read_text() + "\nLocal repair instructions.\n")
    result = invoke(worker, root, "upgrade", "plan", str(worker))
    assert result.returncode == 0, result.stderr
    path = Path(result.stdout.rsplit("Plan: ", 1)[1].strip())
    return path, json.loads(path.read_text())


def test_plan_preserves_settings_and_reports_local_edits(
    worker: Path, predecessor: Path, tmp_path: Path
) -> None:
    path, plan = prepare(worker, predecessor, tmp_path)
    assert plan["from_version"] == "0.1.0"
    assert plan["to_version"] == "0.2.0"
    assert plan["checks"] == ["config-check", "doctor", "check"]
    assert plan["files"]["guides/repair/SKILL.md"]["action"] == "conflict"
    assert plan["files"]["notes/State.md"]["action"] == "keep"
    assert plan["files"][".worker/lint.toml"]["action"] == "keep"
    change = plan["files"]["worker.toml"]
    original = (tmp_path / "worker.toml").read_bytes()
    replacement = (path.parent / "blobs" / change["after"]["sha256"]).read_bytes()
    assert replacement == original.replace(b'"0.1.0"', b'"0.2.0"', 1)
    assert hashlib.sha256(original).hexdigest() == change["before"]["sha256"]
    report = (path.parent / "diff.txt").read_text()
    assert '-runtime = "0.1.0"' in report
    assert '+runtime = "0.2.0"' in report
    assert '"conflict" guides/repair/SKILL.md' in report
    baseline_present = (tmp_path / ".worker/manifest.json").exists()
    expected = "manifest" if baseline_present else "reconstructed"
    assert plan["baseline"] == expected
    assert invoke(predecessor, tmp_path, "config-check").returncode == 0


def test_plan_rejects_unsupported_release(worker: Path, predecessor: Path, tmp_path: Path) -> None:
    assert invoke(predecessor, tmp_path, "init").returncode == 0
    result = invoke(worker, tmp_path, "upgrade", "plan", str(predecessor))
    assert result.returncode == 2
    assert "release must be 0.2.0" in result.stderr
    assert not (tmp_path / ".worker/runtime/upgrade").exists()


def test_plan_tracks_custom_configuration_paths(
    worker: Path, predecessor: Path, tmp_path: Path
) -> None:
    assert invoke(predecessor, tmp_path, "init").returncode == 0
    lint = tmp_path / ".worker/lint.toml"
    custom = tmp_path / "project-lint.toml"
    lint.rename(custom)
    config = tmp_path / "worker.toml"
    config.write_text(config.read_text().replace(".worker/lint.toml", "project-lint.toml"))
    before = custom.read_bytes()
    result = invoke(worker, tmp_path, "upgrade", "plan", str(worker))
    assert result.returncode == 0, result.stderr
    path = Path(result.stdout.rsplit("Plan: ", 1)[1].strip())
    plan = json.loads(path.read_text())
    change = plan["files"]["project-lint.toml"]
    assert change["action"] == "keep"
    assert change["before"]["sha256"] == hashlib.sha256(before).hexdigest()
    assert change["before"] == change["after"]
    assert custom.read_bytes() == before
    assert not lint.exists()
