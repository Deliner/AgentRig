import json
import subprocess
from pathlib import Path

import pytest
from test_delegate_mcp import client, initialize, tool
from test_delegate_run import call, consumer, terminal


def code_consumer(worker: Path, root: Path, monkeypatch: pytest.MonkeyPatch, script: str) -> str:
    consumer(worker, root, monkeypatch, script)
    (root / "src").mkdir()
    (root / "src/value.txt").write_text("before\n")
    commands = [
        ["init", "-q", "-b", "main"],
        ["add", "src"],
        ["-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-qm", "input"],
    ]
    for args in commands:
        subprocess.run(["git", *args], cwd=root, check=True, capture_output=True)
    base = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    config = root / "delegate.yaml"
    config.write_text(
        config.read_text()
        .replace('mode: "artifacts"', 'mode: "code"')
        .replace("timeout_seconds: 2", "timeout_seconds: 60")
        .replace("    programs:", '    programs:\n      python3: "/usr/bin/python3"')
    )
    code_request(root, base)
    return base


def code_request(root: Path, base: str) -> None:
    path = root / "request.json"
    request = json.loads(path.read_text())
    request["revision"] = base
    request["contract"]["artifacts"] = {}
    request["contract"]["changes"] = {
        "write_paths": ["src/value.txt"],
        "checks": {
            "value": [
                "python3",
                "-B",
                "-c",
                "from pathlib import Path; assert Path('src/value.txt').read_text() == 'after\\n'",
            ]
        },
    }
    path.write_text(json.dumps(request))


SCRIPT = "test ! -e /project/.git\nprintf 'after\\n' > /project/src/value.txt\nprintf '{\"ok\":true}' > /work/result.json\n"


def test_code_mcp_returns_checked_patch_without_changing_checkout(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    base = code_consumer(worker, tmp_path, monkeypatch, SCRIPT)
    with client(worker, tmp_path) as process:
        initialize(process)
        started = tool(
            process, "delegate_start", json.loads((tmp_path / "request.json").read_text())
        )
        identifier = started["structuredContent"]["run_id"]
    result = terminal(worker, tmp_path, identifier)
    assert result["outcome"] == "PASS", result
    assert result["code"]["base"] == base
    assert result["code"]["verified"] is True
    assert result["code"]["checks"]["value"]["exit_code"] == 0
    assert (tmp_path / "src/value.txt").read_text() == "before\n"
    directory = tmp_path / ".agentrig/runtime/jobs" / identifier
    applied = subprocess.run(
        ["git", "apply", "--check", str(directory / "change.patch")],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
    )
    assert applied.returncode == 0, applied.stderr
    assert not (directory / "private").exists()


@pytest.mark.parametrize(
    "check",
    [
        ("raise SystemExit(3)", 3),
        ("from pathlib import Path; Path('src/value.txt').write_text('forbidden')", 1),
    ],
)
def test_failed_code_check_retains_patch_and_protects_checkout(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, check: tuple[str, int]
) -> None:
    code_consumer(worker, tmp_path, monkeypatch, SCRIPT)
    path = tmp_path / "request.json"
    request = json.loads(path.read_text())
    source, expected = check
    request["contract"]["changes"]["checks"]["value"] = ["python3", "-c", source]
    path.write_text(json.dumps(request))
    identifier = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    result = terminal(worker, tmp_path, identifier)
    assert result["outcome"] == "ERROR", result
    assert result["code"]["verified"] is False
    assert result["code"]["checks"]["value"]["exit_code"] == expected
    assert (tmp_path / ".agentrig/runtime/jobs" / identifier / "change.patch").is_file()
    assert (tmp_path / "src/value.txt").read_text() == "before\n"


def test_code_request_requires_revision(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    code_consumer(worker, tmp_path, monkeypatch, SCRIPT)
    path = tmp_path / "request.json"
    request = json.loads(path.read_text())
    del request["revision"]
    path.write_text(json.dumps(request))
    result = subprocess.run(
        [worker, "delegate", "--root", tmp_path, "start", "delegate.yaml", "request.json"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 2
    assert "requires revision" in result.stderr


@pytest.mark.parametrize("cancel_first", [False, True])
def test_concurrent_code_runs_preserve_each_other(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, cancel_first: bool
) -> None:
    code_consumer(worker, tmp_path, monkeypatch, "/tools/sleep 1\n" + SCRIPT)
    first = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    second = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    assert first != second
    if cancel_first:
        assert call(worker, tmp_path, "cancel", first)["outcome"] == "CANCELLED"
    else:
        assert terminal(worker, tmp_path, first)["outcome"] == "PASS"
    assert terminal(worker, tmp_path, second)["outcome"] == "PASS"
    assert (tmp_path / "src/value.txt").read_text() == "before\n"
    for identifier in (first, second):
        assert not (tmp_path / ".agentrig/runtime/jobs" / identifier / "private").exists()
