import json
import os
import subprocess
from pathlib import Path


def resources(root: Path) -> None:
    (root / "config.yaml").write_text(
        "schema_version: 1\nrunner:\n  runtime_root: runtime\n  report_root: reports\n"
        "  isolation: bubblewrap\n  parallelism: 1\n  timeout_seconds: 5\n  format_attempts: 2\n"
        "reviewers:\n  critic:\n    model: test\n    reasoning_effort: high\n    prompt: prompt.md\n"
        "tools:\n  review_code:\n    description: Review\n    reviewers: [critic]\n"
        "    project_config: project.yaml\n"
    )
    (root / "project.yaml").write_text(
        "schema_version: 1\nrepository:\n  visible_paths: ['src/**']\n  contract_paths: []\n"
        "review:\n  contract: contract.json\n"
    )
    requirement = dict(id="C-1", text="Value is one", reviewers=["critic"], allow_na=False)
    (root / "contract.json").write_text(
        json.dumps(dict(schema_version=1, requirements=[requirement]))
    )
    (root / "prompt.md").write_text("Check the assigned requirement.")
    (root / "auth").mkdir()
    (root / "auth/auth.json").write_text("{}")
    source = Path(__file__).parents[2] / "worker/review/tests/support/critic.py"
    (root / "codex").write_bytes(source.read_bytes())
    (root / "codex").chmod(0o755)
    (root / "codex-code-mode-host").write_text("unused")


def test_worker_embeds_review_runner_and_stop_hook(worker: Path, tmp_path: Path) -> None:
    resources(tmp_path)
    project = tmp_path / "project"
    (project / "src").mkdir(parents=True)
    (project / "src/value.py").write_text("value = 1\n")
    for args in [
        ["init", "-q"],
        ["add", "."],
        ["-c", "user.name=Test", "-c", "user.email=test@example.com", "commit", "-qm", "candidate"],
    ]:
        subprocess.run(["git", *args], cwd=project, check=True)
    request = dict(root=str(project), base="HEAD", candidate="HEAD", tool="review_code")
    (tmp_path / "request.json").write_text(json.dumps(request))
    result = subprocess.run(
        [
            str(worker),
            "review",
            "run",
            str(tmp_path / "config.yaml"),
            str(tmp_path / "request.json"),
        ],
        env=dict(
            os.environ, REVIEW_CODEX_BIN=str(tmp_path / "codex"), CODEX_HOME=str(tmp_path / "auth")
        ),
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    report = json.loads(result.stdout)
    assert report["verdict"] == "PASS", report
    assert report["roles"][0]["format_attempts"] == 1
    assert report["cleanup_error"] is None
    assert not (tmp_path / "runtime" / report["run_id"]).exists()


def test_worker_exposes_configured_mcp_tools(worker: Path, tmp_path: Path) -> None:
    resources(tmp_path)
    messages = [
        dict(jsonrpc="2.0", id=1, method="initialize", params=dict(protocolVersion="2025-11-25")),
        dict(jsonrpc="2.0", id=2, method="tools/list"),
    ]
    result = subprocess.run(
        [str(worker), "review", "mcp", str(tmp_path / "config.yaml")],
        input="".join(json.dumps(message) + "\n" for message in messages),
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    responses = [json.loads(line) for line in result.stdout.splitlines()]
    assert responses[1]["result"]["tools"][0]["name"] == "review_code"


def claude_registered_services(root: Path) -> None:
    settings = json.loads((root / ".claude/settings.json").read_text())
    command = settings["hooks"]["PreToolUse"][0]["hooks"][0]["command"]
    event = dict(
        hook_event_name="PreToolUse",
        tool_name="Bash",
        tool_input={"command": "echo bypass"},
        cwd=str(root),
    )
    hook = subprocess.run(
        ["sh", "-c", command],
        cwd=root,
        input=json.dumps(event),
        text=True,
        capture_output=True,
        timeout=20,
    )
    assert hook.returncode == 0, hook.stderr
    assert json.loads(hook.stdout)["hookSpecificOutput"]["permissionDecision"] == "deny"
    server = json.loads((root / ".mcp.json").read_text())["mcpServers"]["worker_review"]
    messages = [
        dict(jsonrpc="2.0", id=1, method="initialize", params={"protocolVersion": "2025-11-25"}),
        dict(jsonrpc="2.0", id=2, method="tools/list"),
    ]
    mcp = subprocess.run(
        [server["command"], *server["args"]],
        cwd=root,
        input="".join(json.dumps(message) + "\n" for message in messages),
        text=True,
        capture_output=True,
        timeout=20,
    )
    assert mcp.returncode == 0, mcp.stderr
    assert {tool["name"] for tool in json.loads(mcp.stdout.splitlines()[1])["result"]["tools"]} == {
        "review_code",
        "review_research",
    }


def claude_consumer_settings(root: Path) -> tuple[Path, Path]:
    settings_path = root / ".claude/settings.json"
    settings = json.loads(settings_path.read_text())
    settings["model"] = "consumer-model"
    settings_path.write_text(json.dumps(settings))
    settings_path.chmod(0o600)
    mcp_path = root / ".mcp.json"
    mcp_path.write_text(
        json.dumps(
            {
                "mcpServers": {
                    "unrelated": {"command": "custom", "env": {"TOKEN": "consumer-secret"}}
                }
            }
        )
    )
    return settings_path, mcp_path
