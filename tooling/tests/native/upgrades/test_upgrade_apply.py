import hashlib
import json
import re
import shutil
import subprocess
import tomllib
from pathlib import Path

import yaml

from .test_upgrade_plan import invoke, migrated_config, prepare


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
    assert (tmp_path / ".worker/bin/agentrig").is_file()
    assert not (tmp_path / ".worker/bin/discipline-worker").exists()
    skill = "guides/repair/SKILL.md"
    receipt = json.loads((tmp_path / ".worker/manifest.json").read_text())
    assert receipt["package_version"] == "0.3.0"
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
    assert not (tmp_path / ".worker/bin/agentrig").exists()


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
    original = (
        adapter.read_text().replace(".worker/bin/discipline-worker", ".worker/bin/agentrig")
        + "\n# Local adapter note.\n"
    )
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
    assert yaml.safe_load((tmp_path / "agentrig.yaml").read_text()) == migrated_config(config)
    assert not (tmp_path / "worker.toml").exists()
    assert invoke(worker, tmp_path, "upgrade", "rollback").returncode == 0
    assert (tmp_path / "worker.toml").read_bytes() == config
    assert (tmp_path / ".worker/lint.toml").read_bytes() == lint
    assert not (tmp_path / ".worker/lint.yaml").exists()
    assert not (tmp_path / "agentrig.yaml").exists()


def test_review_configuration_migrates_with_its_projects(
    worker: Path, predecessor: Path, tmp_path: Path
) -> None:
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    assert invoke(predecessor, tmp_path, "init", "--review", "true").returncode == 0
    assert invoke(predecessor, tmp_path, "setup").returncode == 0
    settings = tmp_path / ".codex/config.toml"
    settings.write_text(
        'model = "keep-consumer-model"\n# Consumer preferences\n' + settings.read_text()
    )
    original_settings = settings.read_bytes()
    config = tmp_path / ".worker/review/config/review.toml"
    config.write_text(config.read_text().replace("gpt-5.6-luna", "migration-test-model"))
    before = config.read_bytes()
    expected = tomllib.loads(before.decode())
    result = invoke(worker, tmp_path, "upgrade", "plan", str(worker))
    assert result.returncode == 0, result.stderr
    path = Path(result.stdout.rsplit("Plan: ", 1)[1].strip())
    applied = invoke(worker, tmp_path, "upgrade", "apply", str(path))
    assert applied.returncode == 0, applied.stdout + applied.stderr
    for tool in expected["tools"].values():
        old = config.parent / tool["project_config"]
        tool["project_config"] = str(Path(tool["project_config"]).with_suffix(".yaml"))
        assert not old.exists()
        assert (config.parent / tool["project_config"]).is_file()
    assert yaml.safe_load(config.with_suffix(".yaml").read_text()) == expected
    assert not config.exists()
    assert invoke(worker, tmp_path, "review", "config-check").returncode == 0
    assert registered_tools(tmp_path) == {"review_code", "review_research"}
    assert tomllib.loads(settings.read_text())["model"] == "keep-consumer-model"
    assert invoke(worker, tmp_path, "upgrade", "rollback").returncode == 0
    assert config.read_bytes() == before
    assert not config.with_suffix(".yaml").exists()
    assert settings.read_bytes() == original_settings


def registered_tools(root: Path) -> set[str]:
    settings = tomllib.loads((root / ".codex/config.toml").read_text())
    server = settings["mcp_servers"]["worker_review"]
    messages = [
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {"protocolVersion": "2025-11-25"},
        },
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
    ]
    result = subprocess.run(
        [server["command"], *server["args"]],
        cwd=root,
        input="".join(json.dumps(message) + "\n" for message in messages),
        text=True,
        capture_output=True,
        timeout=10,
        check=True,
    )
    return {tool["name"] for tool in json.loads(result.stdout.splitlines()[1])["result"]["tools"]}


def test_disabled_lint_migration_does_not_require_removed_policy(
    worker: Path, predecessor: Path, tmp_path: Path
) -> None:
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    assert invoke(predecessor, tmp_path, "init").returncode == 0
    config = tmp_path / "worker.toml"
    source = config.read_text().replace("lint = true", "lint = false")
    source, count = re.subn(r'\[\[checks\]\]\nid = "lint"\n.*?(?=\n\[|\Z)', "", source, flags=re.S)
    assert count == 1
    config.write_text(source)
    (tmp_path / ".worker/lint.toml").unlink()
    assert invoke(predecessor, tmp_path, "config-check").returncode == 0
    result = invoke(worker, tmp_path, "upgrade", "plan", str(worker))
    assert result.returncode == 0, result.stderr
    path = Path(result.stdout.rsplit("Plan: ", 1)[1].strip())
    applied = invoke(worker, tmp_path, "upgrade", "apply", str(path))
    assert applied.returncode == 0, applied.stdout + applied.stderr
    assert not (tmp_path / ".worker/lint.yaml").exists()


def external_review_materials(root: Path) -> tuple[Path, bytes]:
    runner = root / ".worker/review/config/review.toml"
    settings = tomllib.loads(runner.read_text())
    reference = settings["tools"]["review_code"]["project_config"]
    project = runner.parent / reference
    source = project.read_text()
    contract = tomllib.loads(source)["review"]["contract"]
    external = root.parent / f"{root.name}-materials"
    external.mkdir()
    (external / "contract.json").write_bytes((project.parent / contract).read_bytes())
    (external / "project.toml").write_text(
        source.replace(f'contract = "{contract}"', 'contract = "contract.json"')
    )
    runner.write_text(
        runner.read_text().replace(
            f'project_config = "{reference}"',
            f"project_config = {json.dumps(str(external / 'project.toml'))}",
        )
    )
    external_review_resources(root, external)
    return external, runner.read_bytes()


def external_review_resources(root: Path, external: Path) -> None:
    runner = root / ".worker/review/config/review.toml"
    settings = tomllib.loads(runner.read_text())
    prompt = settings["reviewers"]["requirements"]["prompt"]
    (external / "prompt.md").write_bytes((runner.parent / prompt).read_bytes())
    runner.write_text(
        runner.read_text().replace(
            f'prompt = "{prompt}"',
            f"prompt = {json.dumps(str(external / 'prompt.md'))}",
        )
    )
    project = runner.parent / settings["tools"]["review_research"]["project_config"]
    reference = tomllib.loads(project.read_text())["review"]["contract"]
    (external / "research.json").write_bytes((project.parent / reference).read_bytes())
    project.write_text(
        project.read_text().replace(
            f'contract = "{reference}"',
            f"contract = {json.dumps(str(external / 'research.json'))}",
        )
    )


def test_external_review_materials_are_imported_without_modifying_source(
    worker: Path, predecessor: Path, tmp_path: Path
) -> None:
    assert invoke(predecessor, tmp_path, "init", "--review", "true").returncode == 0
    assert invoke(predecessor, tmp_path, "setup").returncode == 0
    external, original = external_review_materials(tmp_path)
    material_path = ".worker/review/config/projects/research.toml"
    material_before = (tmp_path / material_path).read_bytes()
    (external / "contract.json").chmod(0o755)
    source = snapshot(external, ["project.toml", "contract.json", "prompt.md", "research.json"])
    assert invoke(predecessor, tmp_path, "review", "config-check").returncode == 0
    result = invoke(worker, tmp_path, "upgrade", "plan", str(worker))
    assert result.returncode == 0, result.stdout + result.stderr
    path = Path(result.stdout.rsplit("Plan: ", 1)[1].strip())
    plan = json.loads(path.read_text())
    imported = []
    for name in plan["files"]:
        selected = name.startswith(".worker/inputs/")
        if selected:
            imported.append(name)
    assert len(imported) == 4
    assert all(name in plan["manifest"]["files"] for name in imported)
    applied = invoke(worker, tmp_path, "upgrade", "apply", str(path))
    assert applied.returncode == 0, applied.stdout + applied.stderr
    assert snapshot(external, list(source)) == source
    for name in imported:
        executable = bool((tmp_path / name).stat().st_mode & 0o111)
        assert executable == plan["manifest"]["files"][name]["executable"]
    shutil.rmtree(external)
    checked = invoke(worker, tmp_path, "review", "config-check")
    assert checked.returncode == 0, checked.stdout + checked.stderr
    rolled = invoke(worker, tmp_path, "upgrade", "rollback")
    assert rolled.returncode == 0, rolled.stdout + rolled.stderr
    assert (tmp_path / ".worker/review/config/review.toml").read_bytes() == original
    assert (tmp_path / material_path).read_bytes() == material_before
    assert all(not (tmp_path / name).exists() for name in imported)


def test_custom_git_adapter_migrates_without_keeping_a_retired_executable(
    worker: Path, predecessor: Path, tmp_path: Path
) -> None:
    assert invoke(predecessor, tmp_path, "init").returncode == 0
    assert invoke(predecessor, tmp_path, "setup").returncode == 0
    adapter = tmp_path / ".worker/hooks/reference-transaction"
    original = (
        adapter.read_text() + '\n# Preserve history: exec "$root/.worker/bin/discipline-worker"\n'
    )
    adapter.write_text(original)
    result = invoke(worker, tmp_path, "upgrade", "plan", str(worker))
    assert result.returncode == 0, result.stdout + result.stderr
    path = Path(result.stdout.rsplit("Plan: ", 1)[1].strip())
    plan = json.loads(path.read_text())
    change = plan["files"][".worker/hooks/reference-transaction"]
    assert change["action"] == "conflict"
    change["resolution"] = "keep"
    path.write_text(json.dumps(plan))
    rejected = invoke(worker, tmp_path, "upgrade", "apply", str(path))
    assert rejected.returncode == 2
    assert "still references the retired executable" in rejected.stderr
    assert adapter.read_text() == original
    assert (tmp_path / ".worker/bin/discipline-worker").is_file()
    assert not (tmp_path / ".worker/bin/agentrig").exists()
    change["resolution"] = "replace"
    path.write_text(json.dumps(plan))
    applied = invoke(worker, tmp_path, "upgrade", "apply", str(path))
    assert applied.returncode == 0, applied.stdout + applied.stderr
    assert adapter.read_text() == original.replace(
        ".worker/bin/discipline-worker", ".worker/bin/agentrig", 1
    )
    verify_reference_adapter(adapter, tmp_path)
    assert invoke(worker, tmp_path, "upgrade", "rollback").returncode == 0
    assert adapter.read_text() == original


def verify_reference_adapter(adapter: Path, root: Path) -> None:
    probe = subprocess.run(
        [str(adapter), "committed"],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
        timeout=10,
    )
    assert probe.returncode == 0, probe.stdout + probe.stderr
