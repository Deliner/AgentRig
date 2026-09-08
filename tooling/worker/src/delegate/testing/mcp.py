import json
import subprocess
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path
from typing import Any


@contextmanager
def client(worker: Path, root: Path) -> Iterator[subprocess.Popen[str]]:
    process = subprocess.Popen(
        [str(worker), "delegate", "--root", str(root), "mcp", "delegate.yaml"],
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
