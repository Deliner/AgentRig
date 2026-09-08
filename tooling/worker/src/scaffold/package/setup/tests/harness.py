import json
import subprocess
from pathlib import Path


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
