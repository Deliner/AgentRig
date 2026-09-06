import json
import subprocess
import time
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path
from typing import Any

import pytest
from test_delegate_run import consumer


@contextmanager
def client(worker: Path, root: Path) -> Iterator[subprocess.Popen[str]]:
    process = subprocess.Popen(
        [str(worker), "delegate", "--root", str(root), "mcp", "delegate.toml"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    try:
        yield process
    finally:
        process.communicate(timeout=10)
        assert process.returncode == 0


def rpc(process: subprocess.Popen[str], method: str, params: dict[str, Any]) -> dict[str, Any]:
    assert process.stdin is not None and process.stdout is not None
    process.stdin.write(
        json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}) + "\n"
    )
    process.stdin.flush()
    value: dict[str, Any] = json.loads(process.stdout.readline())
    return value


def initialize(process: subprocess.Popen[str]) -> None:
    value = rpc(
        process,
        "initialize",
        {
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "worker-test", "version": "1"},
        },
    )
    assert value["result"]["serverInfo"]["name"] == "worker-delegation"
    assert process.stdin is not None
    process.stdin.write('{"jsonrpc":"2.0","method":"notifications/initialized"}\n')
    process.stdin.flush()


def tool(process: subprocess.Popen[str], name: str, arguments: dict[str, Any]) -> dict[str, Any]:
    response = rpc(process, "tools/call", {"name": name, "arguments": arguments})
    assert "error" not in response, response
    result: dict[str, Any] = response["result"]
    return result


def test_mcp_catalog_and_argument_validation(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    consumer(worker, tmp_path, monkeypatch, "exit 0\n")
    with client(worker, tmp_path) as process:
        assert rpc(process, "tools/list", {})["error"]["code"] == -32000
        initialize(process)
        tools = rpc(process, "tools/list", {})["result"]["tools"]
        assert {item["name"] for item in tools} == {
            "delegate_start",
            "delegate_status",
            "delegate_result",
            "delegate_cancel",
        }
        indexed = {item["name"]: item for item in tools}
        start = indexed["delegate_start"]
        assert start["inputSchema"]["properties"]["profile"]["enum"] == ["fixture"]
        request = json.loads((tmp_path / "request.json").read_text())
        request["profile"] = "unknown"
        invalid = rpc(process, "tools/call", {"name": "delegate_start", "arguments": request})
        assert invalid["error"]["code"] == -32602
        assert not (tmp_path / ".worker/runtime/jobs").exists()


def test_mcp_reconnect_keeps_run_and_artifact_result(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    consumer(
        worker,
        tmp_path,
        monkeypatch,
        "/tools/sleep 0.2\nprintf artifact > /work/asset.txt\nprintf '{\"ok\":true}' > /work/result.json\n",
    )
    request = json.loads((tmp_path / "request.json").read_text())
    with client(worker, tmp_path) as process:
        initialize(process)
        started = tool(process, "delegate_start", request)
        identifier = started["structuredContent"]["run_id"]
    with client(worker, tmp_path) as process:
        initialize(process)
        deadline = time.monotonic() + 10
        value = tool(process, "delegate_result", {"run_id": identifier})
        while value["structuredContent"]["outcome"] == "RUNNING" and time.monotonic() < deadline:
            time.sleep(0.05)
            value = tool(process, "delegate_result", {"run_id": identifier})
        assert value["isError"] is False, value
        assert value["structuredContent"]["outcome"] == "PASS", value
        assert (
            value["structuredContent"]["report"]["result"]["artifacts"]["asset.txt"]["bytes"] == 8
        )
    assert len(list((tmp_path / ".worker/runtime/jobs").iterdir())) == 1


def test_mcp_cancel_reports_failure_and_cleans_task(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    consumer(worker, tmp_path, monkeypatch, "/tools/sleep 60\n")
    with client(worker, tmp_path) as process:
        initialize(process)
        started = tool(
            process, "delegate_start", json.loads((tmp_path / "request.json").read_text())
        )
        identifier = started["structuredContent"]["run_id"]
        result = tool(process, "delegate_cancel", {"run_id": identifier})
        assert result["isError"] is True
        assert result["structuredContent"]["outcome"] == "CANCELLED", result
        assert result["structuredContent"]["report"]["cleanup_errors"] == []
    assert not (tmp_path / ".worker/runtime/jobs" / identifier / "private").exists()
