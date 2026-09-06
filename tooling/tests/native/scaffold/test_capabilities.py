import copy
import json
import shutil
import subprocess
import tomllib
from pathlib import Path
from typing import Any

import pytest
import yaml
from support import CONFIG, file_contents, invoke, project
from test_review import resources


def test_review_uses_project_capability_configuration(worker: Path, tmp_path: Path) -> None:
    project(tmp_path, CONFIG + '\ncapabilities:\n  review:\n    config: "config.yaml"\n')
    resources(tmp_path)
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 0, result.stderr
    result = invoke(worker, tmp_path, "review", "config-check")
    assert result.returncode == 0, result.stderr
    messages = [
        dict(jsonrpc="2.0", id=1, method="initialize", params=dict(protocolVersion="2025-11-25")),
        dict(jsonrpc="2.0", id=2, method="tools/list"),
    ]
    result = invoke(
        worker,
        tmp_path,
        "review",
        "mcp",
        input="".join(json.dumps(message) + "\n" for message in messages),
    )
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout.splitlines()[1])["result"]["tools"][0]["name"] == "review_code"
    (tmp_path / "prompt.md").unlink()
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "capabilities.review.config" in result.stderr


def test_disabled_lint_requires_consistent_gate(worker: Path, tmp_path: Path) -> None:
    source = CONFIG + "\ncapabilities:\n  lint: false\n"
    project(tmp_path, source)
    (tmp_path / "lint.yaml").unlink()
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 0, result.stderr
    result = invoke(worker, tmp_path, "lint")
    assert result.returncode == 2
    assert "capabilities.lint is disabled" in result.stderr
    (tmp_path / "agentrig.yaml").write_text(
        source + '\nchecks:\n- id: "lint"\n  kind: "lint"\n  skill: "guides/repair/SKILL.md"\n'
    )
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "lint check is configured" in result.stderr


def test_unknown_and_absent_capabilities_are_actionable(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    result = invoke(worker, tmp_path, "review", "config-check")
    assert result.returncode == 2
    assert "review is not enabled" in result.stderr
    (tmp_path / "agentrig.yaml").write_text(CONFIG + "\ncapabilities:\n  unknown: true\n")
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "unknown field `unknown`" in result.stderr


def test_installation_ships_review_resources(worker: Path, tmp_path: Path) -> None:
    result = invoke(worker, tmp_path, "init", "--review", "true", "--skills", "guides")
    assert result.returncode == 0, result.stderr
    installed = tmp_path / ".agentrig/bin/agentrig"
    result = invoke(installed, tmp_path, "review", "config-check")
    assert result.returncode == 0, result.stderr
    messages = [
        dict(jsonrpc="2.0", id=1, method="initialize", params=dict(protocolVersion="2025-11-25")),
        dict(jsonrpc="2.0", id=2, method="tools/list"),
    ]
    result = invoke(
        installed,
        tmp_path,
        "review",
        "mcp",
        input="".join(json.dumps(message) + "\n" for message in messages),
    )
    assert result.returncode == 0, result.stderr
    tools = json.loads(result.stdout.splitlines()[1])["result"]["tools"]
    assert {tool["name"] for tool in tools} == {"review_code", "review_research"}
    receipt = json.loads((tmp_path / ".agentrig/manifest.json").read_text())["files"]
    assert receipt["guides/review-project/SKILL.md"]["ownership"] == "editable"
    assert receipt[".agentrig/review/config/review.yaml"]["ownership"] == "configuration"
    assert receipt[".agentrig/review/prompts/correctness.md"]["ownership"] == "editable"
    assert receipt["AGENTS.md"]["ownership"] == "editable"


@pytest.mark.parametrize(
    "replacement", ['version: "1"', "version: 1\nversion: 1", "version: &schema 1"]
)
def test_root_yaml_is_strict(worker: Path, tmp_path: Path, replacement: str) -> None:
    project(tmp_path, CONFIG.replace("version: 1", replacement, 1))
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "agentrig.yaml" in result.stderr and "ACTION:" in result.stderr


def test_legacy_root_requires_explicit_migration(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    (tmp_path / "agentrig.yaml").unlink()
    (tmp_path / "worker.toml").write_text('version = 1\nruntime = "0.2.0"\n')
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "explicit upgrade" in result.stderr and "no format fallback" in result.stderr


ENVIRONMENT_PROGRAM = """#!/usr/bin/python3
import json
import os
import sys
mode = sys.argv[1]
if mode == 'hook':
    event = json.load(sys.stdin)
    print(json.dumps({'hookSpecificOutput': {'hookEventName': event['hook_event_name'], 'additionalContext': sys.argv[2]}}))
else:
    for line in sys.stdin:
        request = json.loads(line)
        result = {'tools': [{'name': 'probe', 'inputSchema': {'type': 'object'}}], 'credential_received': os.environ.get('TOKEN') == 'fixture-value'}
        print(json.dumps({'jsonrpc': '2.0', 'id': request['id'], 'result': result}), flush=True)
"""


def environment_declaration(worker: Path, root: Path) -> Path:
    source = root / "declaration"
    result = invoke(worker, source, "init")
    assert result.returncode == 0, result.stderr
    skill = source / "guide"
    skill.mkdir()
    (skill / "SKILL.md").write_text(
        "---\nname: guide\ndescription: Guide for the configured environment probe.\n---\n"
        "Use the configured probe service for this task.\n"
    )
    (skill / "support.txt").write_text("portable support")
    program = source / "handler"
    program.write_text(ENVIRONMENT_PROGRAM)
    program.chmod(0o755)
    path = source / "agentrig.yaml"
    config = yaml.safe_load(path.read_text())
    config["environment"] = {
        "skills": ["guide"],
        "programs": {"handler": "handler"},
        "hooks": {
            "guide": {
                "event": "SessionStart",
                "program": "handler",
                "args": ["hook", "literal 'quotes' $(echo injection)"],
                "timeout_seconds": 10,
            }
        },
        "mcp_servers": {
            "probe": {"program": "handler", "args": ["mcp"], "env": {"TOKEN": "PROBE_VALUE"}}
        },
    }
    path.write_text(yaml.safe_dump(config))
    return path


def test_project_environment_installs_portable_skills_hooks_and_mcp(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    declaration = environment_declaration(worker, tmp_path)
    target = tmp_path / "target"
    target.mkdir()
    preview = invoke(worker, target, "setup", "--config", str(declaration), "--preview")
    assert preview.returncode == 0, preview.stderr
    registration = json.loads(preview.stdout)["registrations"]["codex"]["mcp_servers"]
    assert registration["probe"]["env_vars"] == ["PROBE_VALUE"]
    assert file_contents(target) == {}
    applied = invoke(worker, target, "setup", "--config", str(declaration))
    assert applied.returncode == 0, applied.stdout + applied.stderr
    assert (target / ".agents/skills/guide/support.txt").read_text() == "portable support"
    shutil.rmtree(declaration.parent)
    monkeypatch.setenv("PROBE_VALUE", "fixture-value")
    verify_project_environment(target)
    receipt = json.loads((target / ".agentrig/manifest.json").read_text())["files"]
    assert receipt[".agents/skills/guide/support.txt"]["ownership"] == "editable"
    (target / ".agents/skills/guide/support.txt").write_text("local support")
    before = file_contents(target)
    installed = target / ".agentrig/bin/agentrig"
    repeated = invoke(installed, target, "setup")
    assert repeated.returncode == 0, repeated.stdout + repeated.stderr
    assert file_contents(target) == before
    hook_file = target / ".codex/hooks.json"
    hooks = json.loads(hook_file.read_text())
    hooks["hooks"]["SessionStart"].pop()
    hook_file.write_text(json.dumps(hooks))
    assert invoke(installed, target, "doctor").returncode == 1


def verify_project_environment(root: Path) -> None:
    hooks = json.loads((root / ".codex/hooks.json").read_text())["hooks"]
    assert len(hooks["SessionStart"]) == 2
    assert hooks["PreToolUse"][0]["matcher"].startswith("Bash|")
    command = hooks["SessionStart"][1]["hooks"][0]["command"]
    result = subprocess.run(
        ["sh", "-c", command],
        cwd=root,
        input=json.dumps({"hook_event_name": "SessionStart"}),
        text=True,
        capture_output=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    assert (
        json.loads(result.stdout)["hookSpecificOutput"]["additionalContext"]
        == "literal 'quotes' $(echo injection)"
    )
    config = tomllib.loads((root / ".codex/config.toml").read_text())
    server = config["mcp_servers"]["probe"]
    assert server["env_vars"] == ["PROBE_VALUE"]
    result = subprocess.run(
        [server["command"], *server["args"]],
        cwd=root,
        input=json.dumps({"id": 1, "method": "tools/list"}) + "\n",
        text=True,
        capture_output=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    response = json.loads(result.stdout)["result"]
    assert response["tools"][0]["name"] == "probe"
    assert response["credential_received"]


@pytest.mark.parametrize("case", ["unknown-field", "reserved-server", "missing-program"])
def test_invalid_project_environment_preserves_target(
    worker: Path, tmp_path: Path, case: str
) -> None:
    declaration = environment_declaration(worker, tmp_path)
    config = yaml.safe_load(declaration.read_text())
    environment = config["environment"]
    match case:
        case "unknown-field":
            environment["unknown"] = True
        case "reserved-server":
            environment["mcp_servers"]["worker_review"] = environment["mcp_servers"].pop("probe")
        case "missing-program":
            environment["hooks"]["guide"]["program"] = "missing"
    declaration.write_text(yaml.safe_dump(config))
    target = tmp_path / "target"
    target.mkdir()
    result = invoke(worker, target, "setup", "--config", str(declaration), "--preview")
    assert result.returncode == 2, result.stdout + result.stderr
    assert file_contents(target) == {}


@pytest.mark.parametrize("server_name", ["probe", "replacement"])
def test_project_environment_update_and_rollback(
    worker: Path, tmp_path: Path, server_name: str
) -> None:
    declaration = environment_declaration(worker, tmp_path)
    target = tmp_path / "target"
    target.mkdir()
    installed = invoke(worker, target, "setup", "--config", str(declaration))
    assert installed.returncode == 0, installed.stdout + installed.stderr
    settings = target / ".codex/config.toml"
    settings.write_text(settings.read_text() + '\n[mcp_servers.unrelated]\ncommand = "external"\n')
    paths = [
        ".codex/config.toml",
        ".codex/hooks.json",
        "agentrig.yaml",
        ".agents/skills/guide/support.txt",
    ]
    original = {path: (target / path).read_bytes() for path in paths}
    config = yaml.safe_load(declaration.read_text())
    environment = config["environment"]
    environment["mcp_servers"] = {
        server_name: {"program": "handler", "env": {"TOKEN": "NEW_TOKEN"}}
    }
    environment["hooks"]["guide"]["args"] = ["hook", "updated instruction"]
    declaration.write_text(yaml.safe_dump(config))
    (declaration.parent / "guide/support.txt").write_text("updated support")
    plan = environment_update(worker, target, declaration)
    shutil.rmtree(declaration.parent)
    applied = invoke(worker, target, "upgrade", "apply", str(plan))
    assert applied.returncode == 0, applied.stdout + applied.stderr
    servers = tomllib.loads(settings.read_text())["mcp_servers"]
    assert servers["unrelated"] == {"command": "external"}
    assert servers[server_name]["enabled"]
    assert servers[server_name]["env_vars"] == ["NEW_TOKEN"]
    renamed = server_name != "probe"
    if renamed:
        assert not servers["probe"]["enabled"]
    assert (target / ".agents/skills/guide/support.txt").read_text() == "updated support"
    rolled_back = invoke(worker, target, "upgrade", "rollback")
    assert rolled_back.returncode == 0, rolled_back.stdout + rolled_back.stderr
    assert {path: (target / path).read_bytes() for path in paths} == original


def environment_update(worker: Path, target: Path, declaration: Path) -> Path:
    result = invoke(worker, target, "upgrade", "plan", "--config", str(declaration))
    assert result.returncode == 0, result.stdout + result.stderr
    path = Path(result.stdout.rsplit("Plan: ", 1)[1].strip())
    plan = json.loads(path.read_text())
    assert plan["files"][".codex/config.toml"]["action"] == "conflict"
    plan["files"][".codex/config.toml"]["resolution"] = "replace"
    path.write_text(json.dumps(plan))
    return path


def composed_environment_declaration(worker: Path, root: Path) -> Path:
    declaration = environment_declaration(worker, root)
    config = yaml.safe_load(declaration.read_text())
    package = {
        "schema_version": 1,
        "id": "shared-environment",
        "version": "1",
        "configuration": config.pop("environment"),
    }
    (declaration.parent / "shared.yaml").write_text(yaml.safe_dump(package))
    config["packages"] = [{"path": "shared.yaml", "into": "/environment"}]
    config.setdefault("capabilities", {})["delegation"] = {"config": "delegates.yaml"}
    declaration.write_text(yaml.safe_dump(config))
    profile = {
        "frontend": "codex",
        "model": "configured-model",
        "reasoning_effort": "high",
        "mode": "read",
        "prompt": "prompt.md",
        "visible_paths": [],
        "timeout_seconds": 60,
        "credentials": {"env": {"OPENAI_API_KEY": "DELEGATE_TOKEN"}},
    }
    delegates = {
        "schema_version": 1,
        "packages": [
            {"path": "shared.yaml", "into": f"/profiles/{name}"} for name in ["first", "second"]
        ],
        "overrides": ["/profiles/second/hooks/guide/args"],
        "profiles": {
            "first": profile,
            "second": {
                **copy.deepcopy(profile),
                "hooks": {"guide": {"args": ["hook", "second instructions"]}},
            },
        },
    }
    (declaration.parent / "delegates.yaml").write_text(yaml.safe_dump(delegates))
    (declaration.parent / "prompt.md").write_text("Perform the configured task.")
    return declaration


def test_one_resource_package_installs_project_and_distinct_delegate_environments(
    worker: Path, tmp_path: Path
) -> None:
    declaration = composed_environment_declaration(worker, tmp_path)
    target = tmp_path / "target"
    target.mkdir()
    result = invoke(worker, target, "setup", "--config", str(declaration))
    assert result.returncode == 0, result.stdout + result.stderr
    installed = yaml.safe_load((target / "agentrig.yaml").read_text())
    path = target / installed["capabilities"]["delegation"]["config"]
    profiles = yaml.safe_load(path.read_text())["profiles"]
    assert set(profiles) == {"first", "second"}
    assert profiles["first"]["skills"] == profiles["second"]["skills"]
    assert profiles["first"]["programs"] == profiles["second"]["programs"]
    assert (
        profiles["first"]["hooks"]["guide"]["args"] != profiles["second"]["hooks"]["guide"]["args"]
    )
    for profile in profiles.values():
        skill = path.parent / profile["skills"][0]
        assert (skill / "support.txt").read_text() == "portable support"
        assert (path.parent / profile["programs"]["handler"]).stat().st_mode & 0o111
    assert (target / ".agents/skills/guide/support.txt").read_text() == "portable support"
    receipt = json.loads((target / ".agentrig/composition.json").read_text())
    assert receipt["packages"][0]["id"] == "shared-environment"
    assert (
        len(receipt["configurations"][str(declaration.parent / "delegates.yaml")]["packages"]) == 1
    )
    shutil.rmtree(declaration.parent)
    checked = invoke(target / ".agentrig/bin/agentrig", target, "delegate", "config-check")
    assert checked.returncode == 0, checked.stdout + checked.stderr


def test_inspection_validates_and_exposes_all_selected_configurations(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    declaration = composed_environment_declaration(worker, tmp_path)
    resources(declaration.parent)
    config = yaml.safe_load(declaration.read_text())
    config["capabilities"]["review"] = {"config": "config.yaml"}
    declaration.write_text(yaml.safe_dump(config))
    target = tmp_path / "target"
    target.mkdir()
    (target / "agentrig.yaml").write_text("unrelated existing configuration")
    before = file_contents(target)
    monkeypatch.setenv("PROBE_VALUE", "credential-must-not-appear")
    result = invoke(worker, target, "config-inspect", str(declaration))
    assert result.returncode == 0, result.stdout + result.stderr
    assert "credential-must-not-appear" not in result.stdout
    verify_inspection(json.loads(result.stdout), declaration)
    assert file_contents(target) == before
    path = declaration.parent / "delegates.yaml"
    delegates = yaml.safe_load(path.read_text())
    delegates["profiles"]["first"]["unsupported"] = True
    path.write_text(yaml.safe_dump(delegates))
    failed = invoke(worker, target, "config-inspect", str(declaration))
    assert failed.returncode == 2
    assert "unsupported" in failed.stderr
    assert file_contents(target) == before


def verify_inspection(report: dict[str, Any], declaration: Path) -> None:
    config = report["configuration"]
    documents = report["configurations"]
    assert documents[config["paths"]["lint"]]["rules"]
    profiles = documents[config["capabilities"]["delegation"]["config"]]["profiles"]
    assert profiles["first"]["model"] == "configured-model"
    assert profiles["second"]["hooks"]["guide"]["args"] == ["hook", "second instructions"]
    review_path = config["capabilities"]["review"]["config"]
    review = documents[review_path]
    assert review["runner"]["parallelism"] > 0
    assert review["reviewers"]
    for tool in review["tools"].values():
        material = (Path("/") / review_path).parent / tool["project_config"]
        assert documents[str(material.resolve().relative_to("/"))]["repository"]
    composition = report["composition"]
    assert composition["packages"][0]["id"] == "shared-environment"
    assert composition["provenance"]["/environment/skills"] == str(
        declaration.parent / "shared.yaml"
    )
    delegates = composition["configurations"][str(declaration.parent / "delegates.yaml")]
    assert delegates["provenance"]["/profiles/second/hooks/guide/args"] == str(
        declaration.parent / "delegates.yaml"
    )
