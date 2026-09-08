import json
import os
import subprocess
from pathlib import Path

from tooling.worker.review.testing.consumer import resources


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
