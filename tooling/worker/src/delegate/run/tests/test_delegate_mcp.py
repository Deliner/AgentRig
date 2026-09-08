import json
import time
from pathlib import Path

import pytest

from tooling.worker.src.delegate.testing.consumer import consumer
from tooling.worker.src.delegate.testing.mcp import client, initialize, rpc, tool


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
        assert not (tmp_path / ".agentrig/runtime/jobs").exists()


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
    assert len(list((tmp_path / ".agentrig/runtime/jobs").iterdir())) == 1


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
    assert not (tmp_path / ".agentrig/runtime/jobs" / identifier / "private").exists()
