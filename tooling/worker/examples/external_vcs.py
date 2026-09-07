"""Independent protocol-v1 adapter over Mercurial; no AgentRig imports."""

import json
import os
import subprocess
import sys
from collections.abc import Callable
from pathlib import Path
from typing import Any


def hg(*arguments: str) -> bytes:
    environment = dict(os.environ, HGPLAIN="1", HGRCPATH="", HGRCSKIPREPO="1")
    environment.pop("HGPLAINEXCEPT", None)
    result = subprocess.run(["hg", *arguments], env=environment, capture_output=True, check=True)
    return result.stdout


def resolve(arguments: dict[str, Any]) -> str:
    lines = hg("log", "--rev", arguments["reference"], "--template", "{node}\n").splitlines()
    assert len(lines) == 1, "reference must select one revision"
    return lines[0].decode()


def head(arguments: dict[str, Any]) -> str | None:
    assert not arguments
    value = resolve({"reference": "."})
    unborn = value == "0" * 40
    return None if unborn else value


def parents(arguments: dict[str, Any]) -> list[str]:
    lines = hg("log", "--rev", arguments["revision"], "--template", "{p1node}\n{p2node}\n")
    return list(filter(has_revision, lines.decode().splitlines()))


def observe(arguments: dict[str, Any]) -> dict[str, Any]:
    assert not arguments
    return {
        "branch": hg("branch").decode().strip(),
        "revision": head({}) or "",
        "status": hg("status").decode().strip(),
        "merge_in_progress": len(hg("parents", "--template", "{node}\n").splitlines()) > 1,
        "rebase_in_progress": Path(".hg/rebasestate").exists(),
    }


def has_revision(value: str) -> bool:
    return value != "0" * 40


def start_feature(arguments: dict[str, Any]) -> None:
    current = observe({})
    expected = arguments["expected"]
    assert current["branch"] == expected["branch"], "branch changed before feature start"
    assert current["revision"] == expected["revision"], "revision changed before feature start"
    assert not current["status"], "working copy must be clean"
    assert not current["merge_in_progress"], "merge is pending"
    assert not current["rebase_in_progress"], "rebase is pending"
    environment = dict(os.environ, HGPLAIN="1")
    environment.pop("HGRCSKIPREPO", None)
    environment.pop("HGPLAINEXCEPT", None)
    result = subprocess.run(
        ["hg", "branch", "--", arguments["branch"]], env=environment, capture_output=True
    )
    assert result.returncode == 0, result.stderr.decode()


def tree(arguments: dict[str, Any]) -> list[dict[str, str]]:
    entries = json.loads(
        hg("manifest", "--rev", arguments["revision"], "--debug", "--template", "json")
    )
    kinds = {("644", ""): "file", ("755", "*"): "executable", ("644", "@"): "symlink"}
    return [
        {
            "path": entry["path"],
            "kind": kinds[(entry["mode"], entry["type"])],
            "object": entry["hash"],
        }
        for entry in entries
    ]


def read(arguments: dict[str, Any]) -> list[int]:
    return list(hg("cat", "--rev", arguments["revision"], "--", "path:" + arguments["path"]))


def changed_paths(arguments: dict[str, Any]) -> list[str]:
    return paths(
        hg(
            "status",
            "--rev",
            arguments["base"],
            "--rev",
            arguments["candidate"],
            "--modified",
            "--added",
            "--removed",
            "--no-status",
            "--print0",
        )
    )


def working_files(arguments: dict[str, Any]) -> list[str]:
    assert not arguments
    return paths(
        hg("status", "--clean", "--modified", "--added", "--unknown", "--no-status", "--print0")
    )


def paths(data: bytes) -> list[str]:
    return data.decode().rstrip("\0").split("\0") if data else []


def diff(arguments: dict[str, Any]) -> str:
    return hg("diff", "--git", "--rev", arguments["base"], "--rev", arguments["candidate"]).decode()


OPERATIONS: dict[str, Callable[[dict[str, Any]], Any]] = {
    "resolve": resolve,
    "head": head,
    "observe": observe,
    "start-feature": start_feature,
    "parents": parents,
    "tree": tree,
    "read": read,
    "changed-paths": changed_paths,
    "working-files": working_files,
    "diff": diff,
}


def main() -> None:
    request = json.load(sys.stdin)
    assert request["version"] == 1, "unsupported protocol version"
    operation = OPERATIONS.get(request["operation"])
    unsupported = operation is None
    if unsupported:
        print("unsupported operation: " + request["operation"], file=sys.stderr)
        sys.exit(64)
    assert operation is not None
    result = operation(request["arguments"])
    json.dump({"version": 1, "result": result}, sys.stdout)


running_as_program = __name__ == "__main__"
if running_as_program:
    main()
