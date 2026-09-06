import json
import shutil
import tomllib
from pathlib import Path

import yaml

from .test_upgrade_apply import snapshot
from .test_upgrade_plan import invoke


def consumer(worker: Path, root: Path, review: str = "false") -> tuple[Path, Path]:
    source = root / "source"
    source.mkdir()
    result = invoke(worker, source, "init", "--service", "custom rig", "--review", review)
    assert result.returncode == 0, result.stderr
    declaration = source / "agentrig.yaml"
    config = yaml.safe_load(declaration.read_text())
    config["commands"]["hello"] = {"argv": ["python3", "-c", "print('before')"]}
    declaration.write_text(yaml.safe_dump(config))
    target = root / "target"
    target.mkdir()
    result = invoke(worker, target, "setup", "--config", str(declaration))
    assert result.returncode == 0, result.stdout + result.stderr
    return target, declaration


def update(worker: Path, target: Path, source: Path) -> Path:
    result = invoke(worker, target, "upgrade", "plan", "--config", str(source))
    assert result.returncode == 0, result.stdout + result.stderr
    return Path(result.stdout.rsplit("Plan: ", 1)[1].strip())


def test_configuration_update_and_rollback(worker: Path, tmp_path: Path) -> None:
    target, declaration = consumer(worker, tmp_path)
    declaration.write_text(declaration.read_text().replace("print('before')", "print('after')"))
    skill = declaration.parent / "custom rig/skills/repair/SKILL.md"
    skill.write_text(skill.read_text() + "\nNew upstream instruction.\n")
    state = target / "memory/State.md"
    state.write_text(state.read_text().replace("Not recorded.", "Consumer recovery context."))
    path = update(worker, target, declaration)
    plan = json.loads(path.read_text())
    assert plan["service"] == "custom rig"
    assert plan["from_version"] == plan["to_version"] == "0.3.0"
    assert plan["files"]["memory/State.md"]["action"] == "keep"
    assert all(change["action"] != "conflict" for change in plan["files"].values())
    original = snapshot(target, list(plan["files"]))
    lines = (path.parent / "diff.txt").read_text().splitlines()
    assert any(line.startswith("+") and "print('after')" in line for line in lines)
    result = invoke(worker, target, "upgrade", "apply", str(path))
    assert result.returncode == 0, result.stdout + result.stderr
    result = invoke(worker, target, "run", "hello")
    assert result.returncode == 0, result.stderr
    assert "after" in result.stdout
    assert "New upstream instruction." in (target / "custom rig/skills/repair/SKILL.md").read_text()
    assert state.read_bytes() == original["memory/State.md"]
    result = invoke(worker, target, "upgrade", "rollback")
    assert result.returncode == 0, result.stdout + result.stderr
    assert snapshot(target, list(plan["files"])) == original
    assert "before" in invoke(worker, target, "run", "hello").stdout


def test_configuration_update_reports_local_conflicts(worker: Path, tmp_path: Path) -> None:
    target, declaration = consumer(worker, tmp_path)
    skill_path = "custom rig/skills/repair/SKILL.md"
    local = target / skill_path
    local.write_text(local.read_text() + "\nLocal instruction.\n")
    source = declaration.parent / skill_path
    source.write_text(source.read_text() + "\nUpdated upstream instruction.\n")
    path = update(worker, target, declaration)
    plan = json.loads(path.read_text())
    assert plan["files"][skill_path]["action"] == "conflict"
    original = local.read_bytes()
    result = invoke(worker, target, "upgrade", "apply", str(path))
    assert result.returncode == 2
    assert "unresolved conflict" in result.stderr
    assert local.read_bytes() == original
    plan["files"][skill_path]["resolution"] = "keep"
    path.write_text(json.dumps(plan))
    result = invoke(worker, target, "upgrade", "apply", str(path))
    assert result.returncode == 0, result.stdout + result.stderr
    assert local.read_bytes() == original


def test_successive_configuration_updates_keep_recovery(worker: Path, tmp_path: Path) -> None:
    target, declaration = consumer(worker, tmp_path)
    declaration.write_text(declaration.read_text().replace("print('before')", "print('second')"))
    first = update(worker, target, declaration)
    result = invoke(worker, target, "upgrade", "apply", str(first))
    assert result.returncode == 0, result.stdout + result.stderr
    declaration.write_text(declaration.read_text().replace("print('second')", "print('third')"))
    second = update(worker, target, declaration)
    shutil.rmtree(declaration.parent)
    result = invoke(worker, target, "upgrade", "apply", str(second))
    assert result.returncode == 0, result.stdout + result.stderr
    assert "third" in invoke(worker, target, "run", "hello").stdout
    assert invoke(worker, target, "upgrade", "rollback").returncode == 0
    assert "second" in invoke(worker, target, "run", "hello").stdout


def test_stale_update_preserves_previous_recovery(worker: Path, tmp_path: Path) -> None:
    target, declaration = consumer(worker, tmp_path)
    declaration.write_text(declaration.read_text().replace("print('before')", "print('second')"))
    first = update(worker, target, declaration)
    assert invoke(worker, target, "upgrade", "apply", str(first)).returncode == 0
    declaration.write_text(declaration.read_text().replace("print('second')", "print('third')"))
    second = update(worker, target, declaration)
    state = target / "memory/State.md"
    state.write_text(state.read_text() + "\nLocal recovery note.\n")
    result = invoke(worker, target, "upgrade", "apply", str(second))
    assert result.returncode == 2
    assert "file changed since planning" in result.stderr
    journal = json.loads((second.parent.parent / "operation/journal.json").read_text())
    assert journal["plan"] == str(first)
    assert journal["phase"] == "applied"


def test_update_preserves_codex_settings_and_changes_managed_registration(
    worker: Path, tmp_path: Path
) -> None:
    target, declaration = consumer(worker, tmp_path, "true")
    settings = target / ".codex/config.toml"
    settings.write_text('model = "consumer-model"\n# Keep preferences\n' + settings.read_text())
    settings.chmod(0o600)
    original = settings.read_bytes()
    review = declaration.parent / "custom rig/review/config/review.yaml"
    config = yaml.safe_load(review.read_text())
    config["runner"]["timeout_seconds"] = 123
    review.write_text(yaml.safe_dump(config))
    path = update(worker, target, declaration)
    plan = json.loads(path.read_text())
    change = plan["files"][".codex/config.toml"]
    assert change["action"] == "conflict"
    change["resolution"] = "replace"
    path.write_text(json.dumps(plan))
    result = invoke(worker, target, "upgrade", "apply", str(path))
    assert result.returncode == 0, result.stdout + result.stderr
    actual = tomllib.loads(settings.read_text())
    assert actual["model"] == "consumer-model"
    assert "# Keep preferences" in settings.read_text()
    assert actual["mcp_servers"]["worker_review"]["tool_timeout_sec"] == 183
    assert settings.stat().st_mode & 0o777 == 0o600
    assert invoke(worker, target, "upgrade", "rollback").returncode == 0
    assert settings.read_bytes() == original
