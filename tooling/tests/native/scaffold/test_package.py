# DECISION: D023
import hashlib
import json
import shlex
import shutil
import subprocess
import tomllib
from pathlib import Path

import pytest
import yaml
from support import file_contents, invoke, update_config
from test_setup import delegated_project


@pytest.mark.parametrize("frontend", ["codex", "claude-code"])
@pytest.mark.parametrize("custom_endpoint", [True, False])
def test_project_model_and_api_references(
    worker: Path, tmp_path: Path, frontend: str, custom_endpoint: bool
) -> None:
    assert invoke(worker, tmp_path, "init", "--frontend", frontend).returncode == 0
    api = {"key_env": "AGENT_TEST_KEY"}
    if custom_endpoint:
        api["base_url"] = "https://gateway.example/v1"
    agent = {
        "model": "test-model",
        "reasoning_effort": "high",
        "api": api,
    }
    update_config(tmp_path / "agentrig.yaml", agent=agent)
    before = file_contents(tmp_path)
    preview = invoke(worker, tmp_path, "setup", "--preview")
    assert preview.returncode == 0, preview.stderr
    assert json.loads(preview.stdout)["agent"] == agent
    assert file_contents(tmp_path) == before
    result = invoke(worker, tmp_path, "setup")
    assert result.returncode == 0, result.stderr
    verify_project_api(tmp_path, frontend, custom_endpoint)
    before = file_contents(tmp_path)
    assert invoke(worker, tmp_path, "setup").returncode == 0
    assert file_contents(tmp_path) == before
    update_config(tmp_path / "agentrig.yaml", agent={"model": "conflicting-model"})
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "setup")
    assert result.returncode == 2
    assert "setup conflict" in result.stderr and "model" in result.stderr
    assert file_contents(tmp_path) == before


def verify_project_api(root: Path, frontend: str, custom_endpoint: bool) -> None:
    codex = frontend == "codex"
    if codex:
        settings = tomllib.loads((root / ".codex/config.toml").read_text())
        assert settings["model"] == "test-model"
        assert settings["model_reasoning_effort"] == "high"
        provider = settings["model_providers"][settings["model_provider"]]
        assert provider["env_key"] == "AGENT_TEST_KEY"
        endpoint = "https://gateway.example/v1" if custom_endpoint else "https://api.openai.com/v1"
        assert provider["base_url"] == endpoint
        assert provider["wire_api"] == "responses"
        assert provider["requires_openai_auth"] is False
        return
    settings = json.loads((root / ".claude/settings.json").read_text())
    assert settings["model"] == "test-model"
    assert settings["effortLevel"] == "high"
    claude_endpoint = "https://gateway.example/v1" if custom_endpoint else None
    assert settings.get("env", {}).get("ANTHROPIC_BASE_URL") == claude_endpoint
    helper = ["sh", "-c", settings["apiKeyHelper"]]
    key = "fake-'quoted-$(never-execute)-key"
    result = subprocess.run(
        helper, env={"AGENT_TEST_KEY": key}, capture_output=True, text=True, check=False
    )
    assert result.returncode == 0 and result.stdout == key
    missing = subprocess.run(helper, env={}, capture_output=True, text=True, check=False)
    assert missing.returncode != 0 and not missing.stdout
    assert "missing agent.api.key_env reference" in missing.stderr
    assert key not in (root / ".claude/settings.json").read_text()


def test_doctor_observes_registration_and_tools(worker: Path, tmp_path: Path) -> None:
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    assert invoke(worker, tmp_path, "init").returncode == 0
    assert invoke(worker, tmp_path, "doctor").returncode == 0
    changes = [
        (".codex/config.toml", "hooks = true", "hooks = false", "Codex registration"),
        (".codex/hooks.json", "SessionStart", "UnknownEvent", "Codex registration"),
        (".agentrig/hooks/pre-commit", "--staged", "--incorrect", "git hooks"),
        ("agentrig.yaml", "python3", "missing-tool-xyz", "MISSING"),
        ("agentrig.yaml", "runtime: 0.3.0", "runtime: 999.0.0", "project pins"),
    ]
    for name, before, after, expected in changes:
        path = tmp_path / name
        original = path.read_text()
        assert before in original
        path.write_text(original.replace(before, after))
        verify_doctor_failure(worker, tmp_path, expected)
        path.write_text(original)
    hook = tmp_path / ".agentrig/hooks/pre-commit"
    hook.chmod(0o644)
    assert invoke(worker, tmp_path, "doctor").returncode == 1
    hook.chmod(0o755)
    binary = tmp_path / ".agentrig/bin/agentrig"
    binary.write_text("#!/bin/sh\necho agentrig 999.0.0\n")
    result = invoke(worker, tmp_path, "doctor")
    assert result.returncode == 1
    assert "installed binary: MISSING OR INCOMPATIBLE" in result.stdout


def verify_doctor_failure(worker: Path, root: Path, expected: str) -> None:
    result = invoke(worker, root, "doctor")
    assert result.returncode != 0
    assert expected in result.stdout + result.stderr
    assert ".agentrig/skills/repair/SKILL.md" in result.stderr
    command = result.stderr.split("RERUN: ", 1)[1].splitlines()[0]
    repeated = subprocess.run(shlex.split(command), capture_output=True, text=True, check=False)
    assert repeated.returncode == result.returncode
    assert expected in repeated.stdout + repeated.stderr


@pytest.mark.parametrize("case", ["invalid-glob", "file-parent", "generated-parent"])
def test_init_rejects_invalid_layout_before_writing(
    worker: Path, tmp_path: Path, case: str
) -> None:
    args = ["--source", "src/["]
    existing_file_parent = case == "file-parent"
    generated_file_parent = case == "generated-parent"
    if existing_file_parent:
        (tmp_path / ".codex").write_text("user data")
        args = []
    elif generated_file_parent:
        args = ["--skills", ".agentrig/bin/agentrig"]
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "init", *args)
    assert result.returncode == 2
    assert file_contents(tmp_path) == before


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


@pytest.mark.parametrize("language", ["python", "rust"])
def test_custom_service_executes_installed_adapters(
    worker: Path, tmp_path: Path, language: str
) -> None:
    service = "rig space's $cash"
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    result = invoke(worker, tmp_path, "init", "--language", language, "--service", service)
    assert result.returncode == 0, result.stderr
    assert not (tmp_path / ".agentrig").exists()
    assert not (tmp_path / ".worker").exists()
    installed = tmp_path / service / "bin/agentrig"
    assert invoke(installed, tmp_path, "doctor").returncode == 0
    checked = subprocess.run(
        ["just", "config-check"], cwd=tmp_path, capture_output=True, text=True, check=False
    )
    assert checked.returncode == 0, checked.stderr
    hook = json.loads((tmp_path / ".codex/hooks.json").read_text())
    command = hook["hooks"]["SessionStart"][0]["hooks"][0]["command"]
    started = subprocess.run(
        ["sh", "-c", command],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
        input=json.dumps({"hook_event_name": "SessionStart", "session_id": "custom-service"}),
    )
    assert started.returncode == 0, started.stderr
    assert "complexity" in started.stdout
    blocked = subprocess.run(
        [str(tmp_path / service / "hooks/pre-commit")],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
    )
    assert blocked.returncode != 0
    assert "feature" in blocked.stderr
    assert (tmp_path / service / "manifest.json").is_file()
    assert (tmp_path / service / "runtime/reminders").is_dir()


@pytest.mark.parametrize("service", ["../escape", "bad\npath", "{{injection}}"])
def test_invalid_service_leaves_consumer_untouched(
    worker: Path, tmp_path: Path, service: str
) -> None:
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "init", "--service", service)
    assert result.returncode == 2
    assert file_contents(tmp_path) == before


def test_setup_cannot_silently_relocate_an_installation(worker: Path, tmp_path: Path) -> None:
    assert invoke(worker, tmp_path, "init").returncode == 0
    update_config(tmp_path / "agentrig.yaml", paths={"service": "replacement"})
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "setup")
    assert result.returncode == 2
    assert "setup conflict" in result.stderr
    assert file_contents(tmp_path) == before


def external_configuration(worker: Path, root: Path) -> Path:
    source = delegated_project(worker, root)
    result = invoke(worker, source, "setup")
    assert result.returncode == 0, result.stdout + result.stderr
    skill = source / ".agentrig/skills/repair/SKILL.md"
    skill.write_text(skill.read_text() + "\nExternal repair instruction.\n")
    helper = skill.parent / "helper.sh"
    helper.write_text("#!/bin/sh\necho custom-helper\n")
    helper.chmod(0o755)
    agents = source / "agents"
    (agents / "special").mkdir()
    (agents / "special/SKILL.md").write_text("# Specialized skill\nFollow the task contract.\n")
    program = agents / "tool.sh"
    program.write_text("#!/bin/sh\necho custom-program\n")
    program.chmod(0o755)
    profiles = agents / "profiles.yaml"
    config = yaml.safe_load(profiles.read_text())
    config["profiles"]["reader"].update(
        skills=["special"],
        programs={"asset": "tool.sh"},
        hooks={"notify": {"event": "SessionStart", "program": "asset", "timeout_seconds": 10}},
    )
    profiles.write_text(yaml.safe_dump(config))
    package = root / "package.yaml"
    package.write_text(
        "schema_version: 1\nid: commands\nversion: '1'\nconfiguration:\n"
        "  commands:\n    hello:\n      argv: [python3, -c, \"print('external command')\"]\n"
    )
    declaration = source / "agentrig.yaml"
    declaration.write_text(declaration.read_text() + "\npackages:\n- path: ../package.yaml\n")
    return declaration


def test_external_setup_survives_removal_of_its_sources(worker: Path, tmp_path: Path) -> None:
    declaration = external_configuration(worker, tmp_path)
    target = tmp_path / "target"
    target.mkdir()
    before = file_contents(target)
    preview = invoke(worker, target, "setup", "--config", str(declaration), "--preview")
    assert preview.returncode == 0, preview.stderr
    assert file_contents(target) == before
    applied = invoke(worker, target, "setup", "--config", str(declaration))
    assert applied.returncode == 0, applied.stdout + applied.stderr
    verify_external_resources(target)
    before = file_contents(target)
    repeated = invoke(worker, target, "setup", "--config", str(declaration))
    assert repeated.returncode == 0, repeated.stdout + repeated.stderr
    assert file_contents(target) == before
    shutil.rmtree(declaration.parent)
    (tmp_path / "package.yaml").unlink()
    installed = target / ".agentrig/bin/agentrig"
    for arguments in [("config-check",), ("delegate", "config-check"), ("setup",)]:
        result = invoke(installed, target, *arguments)
        assert result.returncode == 0, result.stdout + result.stderr
    receipt = target / ".agentrig/manifest.json"
    expected = json.loads(before[receipt])
    actual = json.loads(receipt.read_text())
    for path, entry in expected["files"].items():
        assert actual["files"].get(path) == entry, path
    assert actual == expected
    assert file_contents(target) == before
    executed = invoke(installed, target, "run", "hello")
    assert executed.returncode == 0, executed.stderr
    assert "external command" in executed.stdout


def verify_external_resources(target: Path) -> None:
    skill = target / ".agentrig/skills/repair/SKILL.md"
    assert "External repair instruction." in skill.read_text()
    assert skill.with_name("helper.sh").stat().st_mode & 0o111
    config = yaml.safe_load((target / "agentrig.yaml").read_text())
    profiles = target / config["capabilities"]["delegation"]["config"]
    reader = yaml.safe_load(profiles.read_text())["profiles"]["reader"]
    assert (profiles.parent / reader["programs"]["asset"]).stat().st_mode & 0o111
    assert (profiles.parent / reader["skills"][0] / "SKILL.md").is_file()
    assert reader["hooks"]["notify"]["program"] == "asset"


def test_imported_review_outputs_are_ignored_but_inputs_remain_visible(
    worker: Path, tmp_path: Path
) -> None:
    declaration = external_configuration(worker, tmp_path)
    target = tmp_path / "target"
    target.mkdir()
    installed = invoke(worker, target, "setup", "--config", str(declaration))
    assert installed.returncode == 0, installed.stdout + installed.stderr
    config = yaml.safe_load((target / "agentrig.yaml").read_text())
    path = target / config["capabilities"]["review"]["config"]
    review = yaml.safe_load(path.read_text())
    for name in ["runtime_root", "report_root"]:
        output = (path.parent / review["runner"][name]).resolve()
        assert output.is_dir()
        artifact = output / "review-evidence.json"
        artifact.write_text("{}\n")
        assert git_ignored(target, artifact)
    assert not git_ignored(target, path)
    for reviewer in review["reviewers"].values():
        assert not git_ignored(target, (path.parent / reviewer["prompt"]).resolve())


def git_ignored(root: Path, path: Path) -> bool:
    result = subprocess.run(["git", "check-ignore", "--quiet", str(path)], cwd=root, check=False)
    assert result.returncode in [0, 1]
    return result.returncode == 0


@pytest.mark.parametrize("identity", ["commands", "review-policy"])
def test_package_identity_conflicts_across_capabilities_preserve_target(
    worker: Path, tmp_path: Path, identity: str
) -> None:
    declaration = nested_packages(worker, tmp_path)
    package = tmp_path / "shared/agents/package.yaml"
    content = yaml.safe_load(package.read_text())
    content["id"] = identity
    package.write_text(yaml.safe_dump(content))
    target = tmp_path / "target"
    target.mkdir()
    (target / "user.txt").write_text("keep this file")
    before = file_contents(target)
    for arguments in [("--preview",), ()]:
        result = invoke(worker, target, "setup", "--config", str(declaration), *arguments)
        assert result.returncode == 2, result.stdout + result.stderr
        assert f"package identity conflict for {identity} across configurations" in result.stderr
        assert file_contents(target) == before


def test_same_package_can_be_shared_across_capabilities(worker: Path, tmp_path: Path) -> None:
    declaration = nested_packages(worker, tmp_path)
    common = tmp_path / "common.yaml"
    common.write_text("schema_version: 1\nid: common\nversion: '1'\nconfiguration: {}\n")
    root = yaml.safe_load(declaration.read_text())
    root["packages"].append({"path": "../common.yaml"})
    declaration.write_text(yaml.safe_dump(root))
    profiles = tmp_path / "shared/agents/profiles.yaml"
    content = yaml.safe_load(profiles.read_text())
    content["packages"].append({"path": "../../common.yaml"})
    profiles.write_text(yaml.safe_dump(content))
    target = tmp_path / "target"
    target.mkdir()
    result = invoke(worker, target, "setup", "--config", str(declaration))
    assert result.returncode == 0, result.stdout + result.stderr
    receipt = json.loads((target / ".agentrig/composition.json").read_text())
    packages = receipt["packages"] + [
        package for item in receipt["configurations"].values() for package in item["packages"]
    ]
    copies = []
    for package in packages:
        common = package["id"] == "common"
        if common:
            copies.append(package)
    assert len(copies) == 2
    assert copies[0] == copies[1]


def packaged_configuration(path: Path, identity: str) -> Path:
    package = path.with_name("package.yaml")
    content = {
        "schema_version": 1,
        "id": identity,
        "version": "1",
        "configuration": yaml.safe_load(path.read_text()),
    }
    package.write_text(yaml.safe_dump(content))
    path.write_text(yaml.safe_dump({"packages": [{"path": "package.yaml"}]}))
    return package


def nested_packages(worker: Path, root: Path) -> Path:
    declaration = external_configuration(worker, root)
    shared = root / "shared"
    shared.mkdir()
    shutil.move(str(declaration.parent / "agents"), shared / "agents")
    shutil.move(str(declaration.parent / ".agentrig/review"), shared / "review")
    shutil.move(str(declaration.parent / ".agentrig/lint.yaml"), shared / "lint.yaml")
    review = shared / "review/config/review.yaml"
    packaged_configuration(review, "review-policy")
    packaged_configuration(review.parent / "projects/code.yaml", "code-materials")
    profiles = shared / "agents/profiles.yaml"
    packaged_configuration(profiles, "workers")
    profiles.write_text(
        profiles.read_text() + "overrides: [/profiles/reader/model]\n"
        "profiles:\n  reader:\n    model: project-model\n"
    )
    lint = shared / "lint.yaml"
    policy = yaml.safe_load(lint.read_text())
    policy["rules"][0].update(warning_skill="fix/SKILL.md", error_skill="fix/SKILL.md")
    lint.write_text(yaml.safe_dump(policy))
    (shared / "fix").mkdir()
    (shared / "fix/SKILL.md").write_text(
        "---\nname: fix\ndescription: Repair the selected package rule.\n---\n"
        "Read the reported rule and correct its cause.\n"
    )
    packaged_configuration(lint, "lint-policy")
    update_config(
        declaration,
        paths={"lint": "../shared/lint.yaml"},
        capabilities={
            "review": {"config": "../shared/review/config/review.yaml"},
            "delegation": {"config": "../shared/agents/profiles.yaml"},
        },
    )
    return declaration


def test_capability_packages_preserve_resource_origins(worker: Path, tmp_path: Path) -> None:
    declaration = nested_packages(worker, tmp_path)
    target = tmp_path / "target"
    target.mkdir()
    result = invoke(worker, target, "setup", "--config", str(declaration))
    assert result.returncode == 0, result.stdout + result.stderr
    root = yaml.safe_load((target / "agentrig.yaml").read_text())
    delegate = target / root["capabilities"]["delegation"]["config"]
    profile = yaml.safe_load(delegate.read_text())["profiles"]["reader"]
    assert profile["model"] == "project-model"
    assert profile["hooks"]["notify"]["event"] == "SessionStart"
    assert (delegate.parent / profile["prompt"]).is_file()
    policy = yaml.safe_load((target / root["paths"]["lint"]).read_text())
    assert "selected package rule" in (target / policy["rules"][0]["error_skill"]).read_text()
    receipt = json.loads((target / ".agentrig/composition.json").read_text())
    configurations = receipt["configurations"]
    packages = [package for item in configurations.values() for package in item["packages"]]
    assert {package["id"] for package in packages} == {
        "workers",
        "review-policy",
        "lint-policy",
        "code-materials",
    }
    shutil.rmtree(tmp_path / "shared")
    shutil.rmtree(declaration.parent)
    for arguments in [("config-check",), ("review", "config-check"), ("delegate", "config-check")]:
        checked = invoke(target / ".agentrig/bin/agentrig", target, *arguments)
        assert checked.returncode == 0, checked.stdout + checked.stderr


@pytest.mark.parametrize("case", ["unsupported-frontend", "changed-package"])
def test_nested_package_errors_preserve_the_consumer(
    worker: Path, tmp_path: Path, case: str
) -> None:
    declaration = nested_packages(worker, tmp_path)
    target = tmp_path / "target"
    target.mkdir()
    package = tmp_path / "shared/agents/package.yaml"
    changed = case == "changed-package"
    if changed:
        installed = invoke(worker, target, "setup", "--config", str(declaration))
        assert installed.returncode == 0, installed.stdout + installed.stderr
        package.write_text(package.read_text() + "# Modified fixed input\n")
        expected = "composition inputs changed"
    else:
        package.write_text(package.read_text().replace("frontend: codex", "frontend: unsupported"))
        expected = "frontend"
    before = file_contents(target)
    result = invoke(worker, target, "setup", "--config", str(declaration), "--preview")
    assert result.returncode == 2, result.stdout + result.stderr
    assert expected in result.stderr
    assert file_contents(target) == before


@pytest.mark.parametrize("case", ["missing-prompt", "linked-skill", "changed-package"])
def test_external_setup_rejects_invalid_inputs_without_writing(
    worker: Path, tmp_path: Path, case: str
) -> None:
    declaration = external_configuration(worker, tmp_path)
    target = tmp_path / "target"
    target.mkdir()
    changed_package = case == "changed-package"
    missing_prompt = case == "missing-prompt"
    if changed_package:
        installed = invoke(worker, target, "setup", "--config", str(declaration))
        assert installed.returncode == 0, installed.stdout + installed.stderr
        package = tmp_path / "package.yaml"
        package.write_text(package.read_text() + "# Updated input\n")
    elif missing_prompt:
        (declaration.parent / "agents/prompt.md").unlink()
    else:
        (declaration.parent / "agents/special/link").symlink_to(tmp_path / "package.yaml")
    before = file_contents(target)
    for arguments in [("--preview",), ()]:
        result = invoke(worker, target, "setup", "--config", str(declaration), *arguments)
        assert result.returncode == 2, result.stdout + result.stderr
        assert file_contents(target) == before
