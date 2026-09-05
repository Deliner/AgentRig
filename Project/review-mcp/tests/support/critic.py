#!/usr/bin/python3
import json
import socket
import sys
import time
from pathlib import Path

MODE = "pass"


def hook():
    with socket.socket(socket.AF_UNIX) as connection:
        connection.connect("/review-bin/control.sock")
        return json.loads(connection.recv(65536))


def isolation():
    assert Path.cwd() == Path("/work")
    for hidden in ["/home/deliner", "/project/.git", "/reviewers", "/codex/config.toml"]:
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
    failure = MODE in ["fail", "late"]
    status = "FAIL" if failure else "PASS"
    late = MODE == "late"
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
                late_finding=MODE == "late",
                previous_omission="Earlier review missed this" if late else "",
            )
        ],
    )
    Path("/work/review.json").write_text(json.dumps(answer))
    hook()


main()
