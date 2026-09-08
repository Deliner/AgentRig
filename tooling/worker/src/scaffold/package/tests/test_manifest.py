# DECISION: D023
import hashlib
import json
from pathlib import Path

from tooling.worker.src.scaffold.testing.consumer import file_contents, invoke


# INVARIANT: I020
def test_installation_manifest_records_ownership(worker: Path, tmp_path: Path) -> None:
    result = invoke(worker, tmp_path, "init", "--skills", "guides", "--memory", "notes")
    assert result.returncode == 0, result.stderr
    manifest = json.loads((tmp_path / ".agentrig/manifest.json").read_text())
    assert manifest["manifest_version"] == 1
    assert manifest["package_version"] == "0.3.0"
    assert manifest["config_schema"] == 1
    entries = manifest["files"]
    assert ".agentrig/manifest.json" not in entries
    assert entries[".agentrig/bin/agentrig"]["ownership"] == "runtime"
    assert entries[".agentrig/.gitignore"]["ownership"] == "asset"
    assert entries["agentrig.yaml"]["ownership"] == "configuration"
    assert entries[".agentrig/lint.yaml"]["ownership"] == "configuration"
    assert entries["guides/repair/SKILL.md"]["ownership"] == "editable"
    assert entries[".agentrig/hooks/pre-commit"]["ownership"] == "editable"
    assert entries["notes/State.md"]["ownership"] == "memory"
    for relative, entry in entries.items():
        path = tmp_path / relative
        assert hashlib.sha256(path.read_bytes()).hexdigest() == entry["sha256"]
        assert bool(path.stat().st_mode & 0o111) == entry["executable"]
    skill = tmp_path / "guides/repair/SKILL.md"
    skill.write_text(skill.read_text() + "\nLocal instruction.\n")
    assert (
        hashlib.sha256(skill.read_bytes()).hexdigest()
        != entries["guides/repair/SKILL.md"]["sha256"]
    )


def test_existing_manifest_is_an_init_collision(worker: Path, tmp_path: Path) -> None:
    path = tmp_path / ".agentrig/manifest.json"
    path.parent.mkdir()
    path.write_text("existing receipt")
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "init")
    assert result.returncode == 2
    assert "collision" in result.stderr
    assert file_contents(tmp_path) == before
