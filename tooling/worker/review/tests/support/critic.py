#!/usr/bin/python3
import json
import os
import subprocess
import sys
import time
from pathlib import Path

MODE = "pass"


def hook():
    output = subprocess.run(
        ["/review-bin/review-runner", "review-hook"], capture_output=True, text=True, check=True
    )
    return json.loads(output.stdout)


def isolation():
    assert Path.cwd() == Path("/work")
    for hidden in [
        "/home/deliner",
        "/project/.git",
        "/project/.hg",
        "/reviewers",
        "/codex/config.toml",
    ]:
        assert not Path(hidden).exists(), hidden
    for protected in [
        "/project/src/value.py",
        "/review-input/contract.json",
        "/review-bin/review-runner",
    ]:
        try:
            Path(protected).write_text("overwritten")
        except OSError:
            continue
        raise AssertionError(protected)
    Path("/work/own-file").write_text("own")
    assert list(Path("/home/critic").iterdir()) == []


def main():
    print(time.monotonic(), flush=True)
    assert "UNDECLARED_SECRET" not in os.environ
    claude = "--print" in sys.argv
    if claude:
        assert os.environ["CLAUDE_CODE_OAUTH_TOKEN"] == "fixture-token"
        assert os.environ["CLAUDE_CONFIG_DIR"] == "/claude"
        assert "CODEX_HOME" not in os.environ
        assert "--bare" not in sys.argv
        settings = json.loads(Path("/review-bin/settings.json").read_text())
        assert settings["hooks"]["Stop"][0]["hooks"][0]["command"].endswith("review-hook")
        assert sys.argv[sys.argv.index("--setting-sources") + 1] == ""
        assert "--strict-mcp-config" in sys.argv
    data = sys.stdin.read().split("Expected identity and contract:\n")[1]
    expected = json.loads(data)
    isolation()
    timeout = MODE == "timeout"
    if timeout:
        time.sleep(30)
    exhausted = MODE == "exhausted"
    if exhausted:
        assert hook()["decision"] == "block"
        assert hook()["continue"] is False
    correction = MODE == "correction"
    if correction:
        assert hook()["decision"] == "block"
    parallel = MODE == "parallel"
    if parallel:
        time.sleep(0.3)
    Path("/work/review.json").write_text(json.dumps(answer(expected)))
    hook()
    cleanup_failure = MODE == "cleanup"
    if cleanup_failure:
        Path("/work").chmod(0)
    print(time.monotonic(), flush=True)


def answer(expected):
    failure = MODE in ["fail", "late"]
    status = "FAIL" if failure else "PASS"
    blocked = MODE in ["blocked", "late-blocked"]
    if blocked:
        status = "BLOCKED"
    late = MODE in ["late", "late-blocked"]
    answer = {key: expected[key] for key in ["run_id", "candidate", "contract_digest", "role"]}
    answer.update(
        verdict=status,
        observations=["Informational"],
        checks=[
            dict(
                contract_id="C-1",
                status=status,
                evidence="src/value.py:1",
                finding="Wrong value" if failure else "",
                minimal_fix="Set value to one" if failure else "",
                late_finding=late,
                previous_omission="Earlier review missed this" if late else "",
            )
        ],
    )
    return answer


main()
