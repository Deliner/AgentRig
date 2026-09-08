import json
import subprocess
import time
from pathlib import Path
from typing import Any

import pytest


def consumer(worker: Path, root: Path, monkeypatch: pytest.MonkeyPatch, script: str) -> None:
    result = subprocess.run(
        [str(worker), "init", "--root", str(root)], capture_output=True, text=True, check=False
    )
    assert result.returncode == 0, result.stderr
    binary = root / "fixture/codex"
    binary.parent.mkdir()
    binary.write_text("#!/bin/sh\nset -eu\n" + script)
    binary.chmod(0o755)
    binary.with_name("codex-code-mode-host").write_text("")
    monkeypatch.setenv("DELEGATE_CODEX_BIN", str(binary))
    monkeypatch.setenv("DELEGATE_FAKE_KEY", "unused-fixture-key")
    monkeypatch.setenv("WORKER_OWNER", "delegate-fixture")
    (root / "prompt.md").write_text("Complete the task.\n")
    (root / "delegate.yaml").write_text(CONFIG)
    request = {
        "profile": "fixture",
        "task": "Generate an artifact",
        "contract": {
            "result_schema": {
                "type": "object",
                "required": ["ok"],
                "properties": {"ok": {"const": True}},
            },
            "artifacts": {"asset.txt": 16},
        },
    }
    (root / "request.json").write_text(json.dumps(request))


CONFIG = """schema_version: 1
profiles:
  fixture:
    frontend: "codex"
    model: "fixture"
    reasoning_effort: "high"
    mode: "artifacts"
    prompt: "prompt.md"
    visible_paths: ["src/**"]
    timeout_seconds: 2
    programs:
      sleep: "/usr/bin/sleep"
    credentials:
      env:
        OPENAI_API_KEY: "DELEGATE_FAKE_KEY"
"""


def select_claude(root: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    path = root / "delegate.yaml"
    path.write_text(
        path.read_text()
        .replace('frontend: "codex"', 'frontend: "claude-code"')
        .replace("OPENAI_API_KEY:", "CLAUDE_CODE_OAUTH_TOKEN:")
    )
    monkeypatch.setenv("DELEGATE_CLAUDE_BIN", str(root / "fixture/codex"))
    (root / "fixture/codex-code-mode-host").unlink()


CLAUDE_RESPONSE = 'printf \'{"is_error":false,"structured_output":{"ok":true}}\\n\'\n'


def call(worker: Path, root: Path, *args: str) -> dict[str, Any]:
    result = subprocess.run(
        [str(worker), "delegate", "--root", str(root), *args],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    value: dict[str, Any] = json.loads(result.stdout)
    return value


def terminal(worker: Path, root: Path, identifier: str) -> dict[str, Any]:
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        value = call(worker, root, "result", identifier)
        done = value["job"]["state"] in {"completed", "cancelled", "interrupted"}
        if done:
            return value
        time.sleep(0.05)
    raise AssertionError("delegate did not finish")
