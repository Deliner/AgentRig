import json
import shutil
from pathlib import Path

import yaml

from .test_upgrade_plan import invoke

PROFILE = """schema_version = 1
[profiles.reader]
frontend = "codex"
model = "migration-model"
reasoning_effort = "high"
mode = "read"
prompt = {prompt}
skills = [{skill}]
visible_paths = ["src/**"]
timeout_seconds = 60
[profiles.reader.programs]
helper = {program}
[profiles.reader.credentials]
codex_auth_file_env = "DELEGATE_AUTH"
[profiles.reader.mcp_servers.helper]
program = "helper"
args = []
"""


def prepare_delegate(predecessor: Path, root: Path) -> tuple[Path, bytes]:
    assert invoke(predecessor, root, "init").returncode == 0
    external = root.parent / f"{root.name}-delegate"
    skill = external / "guide"
    skill.mkdir(parents=True)
    (skill / "SKILL.md").write_text(
        "---\nname: guide\ndescription: Migration task guidance.\n---\n"
    )
    (skill / "support.txt").write_text("selected support")
    (external / "prompt.md").write_text("Perform the selected task.")
    program = external / "helper"
    program.write_text("#!/bin/sh\nprintf 'configured helper'\n")
    program.chmod(0o755)
    source = PROFILE.format(
        prompt=json.dumps(str(external / "prompt.md")),
        skill=json.dumps(str(skill)),
        program=json.dumps(str(program)),
    )
    (root / "profiles.toml").write_text(source)
    config = root / "worker.toml"
    config.write_text(
        config.read_text() + '\n[capabilities.delegation]\nconfig = "profiles.toml"\n'
    )
    checked = invoke(predecessor, root, "config-check")
    assert checked.returncode == 0, checked.stdout + checked.stderr
    installed = invoke(predecessor, root, "setup")
    assert installed.returncode == 0, installed.stdout + installed.stderr
    return external, source.encode()


def test_delegate_migration_preserves_selected_environment_and_guidance(
    worker: Path, predecessor: Path, tmp_path: Path
) -> None:
    external, original = prepare_delegate(predecessor, tmp_path)
    guide = ".worker/skills/delegate-task/SKILL.md"
    local = (tmp_path / guide).read_text() + "\nKeep this consumer instruction.\n"
    (tmp_path / guide).write_text(local)
    result = invoke(worker, tmp_path, "upgrade", "plan", str(worker))
    assert result.returncode == 0, result.stdout + result.stderr
    path = Path(result.stdout.rsplit("Plan: ", 1)[1].strip())
    plan = json.loads(path.read_text())
    for name, change in plan["files"].items():
        conflict = change["action"] == "conflict"
        if conflict:
            custom = name == guide
            change["resolution"] = "keep" if custom else "replace"
    path.write_text(json.dumps(plan))
    applied = invoke(worker, tmp_path, "upgrade", "apply", str(path))
    assert applied.returncode == 0, applied.stdout + applied.stderr
    assert (tmp_path / guide).read_text() == local
    verify_installed_delegate(tmp_path)
    shutil.rmtree(external)
    checked = invoke(worker, tmp_path, "delegate", "config-check")
    assert checked.returncode == 0, checked.stdout + checked.stderr
    rolled = invoke(worker, tmp_path, "upgrade", "rollback")
    assert rolled.returncode == 0, rolled.stdout + rolled.stderr
    assert (tmp_path / "profiles.toml").read_bytes() == original
    assert not (tmp_path / "profiles.yaml").exists()
    assert (tmp_path / guide).read_text() == local


def verify_installed_delegate(root: Path) -> None:
    profile = yaml.safe_load((root / "profiles.yaml").read_text())["profiles"]["reader"]
    receipt = json.loads((root / ".worker/manifest.json").read_text())["files"]
    assert receipt["profiles.yaml"]["ownership"] == "configuration"
    assert receipt[".worker/skills/delegate-task/SKILL.md"]["ownership"] == "editable"
    assert "delegate-task/SKILL.md" in (root / "AGENTS.md").read_text()
    resources = [profile["prompt"], profile["programs"]["helper"], *profile["skills"]]
    for resource in resources:
        assert (root / resource).resolve().is_relative_to(root)
    assert (root / profile["prompt"]).read_text() == "Perform the selected task."
    assert (root / profile["programs"]["helper"]).stat().st_mode & 0o111
    skill = root / profile["skills"][0]
    assert (skill / "support.txt").read_text() == "selected support"
    assert str((skill / "support.txt").relative_to(root)) in receipt
