import hashlib
import json
import shutil
import subprocess
import tomllib
from pathlib import Path

import yaml

from .test_upgrade_plan import invoke, prepare


def test_apply_rejects_unresolved_and_stale_plans(
    worker: Path, predecessor: Path, tmp_path: Path
) -> None:
    path, plan = prepare(worker, predecessor, tmp_path)
    binary = tmp_path / ".worker/bin/discipline-worker"
    original = binary.read_bytes()
    result = invoke(worker, tmp_path, "upgrade", "apply", str(path))
    assert result.returncode == 2
    assert "unresolved conflict" in result.stderr
    assert binary.read_bytes() == original
    plan["files"]["guides/repair/SKILL.md"]["resolution"] = "keep"
    path.write_text(json.dumps(plan))
    state = tmp_path / "notes/State.md"
    state.write_text(state.read_text() + "\nChanged since planning.\n")
    result = invoke(worker, tmp_path, "upgrade", "apply", str(path))
    assert result.returncode == 2
    assert "file changed since planning: notes/State.md" in result.stderr
    assert binary.read_bytes() == original


def test_apply_and_rollback_preserve_local_content(
    worker: Path, predecessor: Path, tmp_path: Path
) -> None:
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    path, plan = prepare(worker, predecessor, tmp_path)
    plan["files"]["guides/repair/SKILL.md"]["resolution"] = "keep"
    path.write_text(json.dumps(plan))
    original = snapshot(tmp_path, list(plan["files"]))
    result = invoke(worker, tmp_path, "upgrade", "apply", str(path))
    assert result.returncode == 0, result.stdout + result.stderr
    assert invoke(worker, tmp_path, "doctor").returncode == 0
    skill = "guides/repair/SKILL.md"
    receipt = json.loads((tmp_path / ".worker/manifest.json").read_text())
    assert receipt["package_version"] == "0.2.0"
    assert receipt["local"][skill] == hashlib.sha256(original[skill]).hexdigest()
    assert (tmp_path / skill).read_bytes() == original[skill]
    for name in ["Plan", "State", "Decisions", "Invariants"]:
        relative = f"notes/{name}.md"
        assert (tmp_path / relative).read_bytes() == original[relative]
    assert invoke(worker, tmp_path, "upgrade", "apply", str(path)).returncode == 0
    rolled = invoke(worker, tmp_path, "upgrade", "rollback")
    assert rolled.returncode == 0, rolled.stderr
    for name, content in original.items():
        assert (tmp_path / name).read_bytes() == content
    assert invoke(predecessor, tmp_path, "config-check").returncode == 0


def snapshot(root: Path, names: list[str]) -> dict[str, bytes]:
    result = {}
    for name in names:
        path = root / name
        exists = path.is_file()
        if exists:
            result[name] = path.read_bytes()
    return result


def test_failed_verification_can_resume(worker: Path, predecessor: Path, tmp_path: Path) -> None:
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    path, plan = prepare(worker, predecessor, tmp_path)
    plan["files"]["guides/repair/SKILL.md"]["resolution"] = "replace"
    path.write_text(json.dumps(plan))
    source = tmp_path / "src"
    source.mkdir()
    broken = source / "test_project.py"
    broken.write_text("def test_value():\n    assert False\n")
    result = invoke(worker, tmp_path, "upgrade", "apply", str(path))
    assert result.returncode != 0
    journal_path = path.parent.parent / "operation/journal.json"
    journal = json.loads(journal_path.read_text())
    assert journal["phase"] == "validating"
    assert journal["checks"]["config-check"] == 0
    assert journal["checks"]["doctor"] == 0
    assert journal["checks"]["check"] != 0
    assert "upgrade apply" in journal["next_action"]
    broken.write_text("def test_value():\n    assert True\n")
    resumed = invoke(worker, tmp_path, "upgrade", "apply", str(path))
    assert resumed.returncode == 0, resumed.stdout + resumed.stderr
    assert json.loads(journal_path.read_text())["phase"] == "applied"
    assert broken.read_text() == "def test_value():\n    assert True\n"


def test_rollback_refuses_to_overwrite_later_edit(
    worker: Path, predecessor: Path, tmp_path: Path
) -> None:
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    path, plan = prepare(worker, predecessor, tmp_path)
    plan["files"]["guides/repair/SKILL.md"]["resolution"] = "replace"
    path.write_text(json.dumps(plan))
    assert invoke(worker, tmp_path, "upgrade", "apply", str(path)).returncode == 0
    changed = tmp_path / "guides/repair/SKILL.md"
    updated = changed.read_text() + "\nNew user instruction after upgrade.\n"
    changed.write_text(updated)
    result = invoke(worker, tmp_path, "upgrade", "rollback")
    assert result.returncode == 2
    assert "file changed after upgrade" in result.stderr
    assert changed.read_text() == updated


def test_kept_adapter_is_explicitly_approved(
    worker: Path, predecessor: Path, tmp_path: Path
) -> None:
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    assert invoke(predecessor, tmp_path, "init").returncode == 0
    adapter = tmp_path / ".worker/hooks/pre-commit"
    original = adapter.read_text() + "\n# Local adapter note.\n"
    adapter.write_text(original)
    result = invoke(worker, tmp_path, "upgrade", "plan", str(worker))
    assert result.returncode == 0, result.stderr
    path = Path(result.stdout.rsplit("Plan: ", 1)[1].strip())
    plan = json.loads(path.read_text())
    change = plan["files"][".worker/hooks/pre-commit"]
    assert change["action"] == "conflict"
    change["resolution"] = "keep"
    path.write_text(json.dumps(plan))
    applied = invoke(worker, tmp_path, "upgrade", "apply", str(path))
    assert applied.returncode == 0, applied.stdout + applied.stderr
    assert adapter.read_text() == original
    assert invoke(worker, tmp_path, "doctor").returncode == 0
    adapter.write_text(original + "# Unreviewed subsequent change.\n")
    assert invoke(worker, tmp_path, "doctor").returncode != 0


def test_rust_consumer_upgrade(worker: Path, predecessor: Path, tmp_path: Path) -> None:
    example = Path(__file__).parents[4] / "tooling/worker/examples/rust"
    shutil.copytree(example, tmp_path, dirs_exist_ok=True)
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    installed = invoke(
        predecessor,
        tmp_path,
        "init",
        "--language",
        "rust",
        "--source",
        "crates/engine",
        "--memory",
        "knowledge",
        "--skills",
        "policies",
    )
    assert installed.returncode == 0, installed.stderr
    config = (tmp_path / "worker.toml").read_bytes()
    lint = (tmp_path / ".worker/lint.toml").read_bytes()
    result = invoke(worker, tmp_path, "upgrade", "plan", str(worker))
    assert result.returncode == 0, result.stderr
    path = Path(result.stdout.rsplit("Plan: ", 1)[1].strip())
    applied = invoke(worker, tmp_path, "upgrade", "apply", str(path))
    assert applied.returncode == 0, applied.stdout + applied.stderr
    assert "test result: ok" in applied.stdout
    assert yaml.safe_load((tmp_path / ".worker/lint.yaml").read_text()) == tomllib.loads(
        lint.decode()
    )
    expected = config.replace(b'"0.1.0"', b'"0.2.0"', 1).replace(b"lint.toml", b"lint.yaml")
    assert (tmp_path / "worker.toml").read_bytes() == expected
    assert invoke(worker, tmp_path, "upgrade", "rollback").returncode == 0
    assert (tmp_path / "worker.toml").read_bytes() == config
    assert (tmp_path / ".worker/lint.toml").read_bytes() == lint
    assert not (tmp_path / ".worker/lint.yaml").exists()
