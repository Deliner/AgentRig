"""Independent protocol-v1 adapter over Mercurial; no AgentRig imports."""

import json
import os
import shlex
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
    write("branch", "--", arguments["branch"])


def initialize(arguments: dict[str, Any]) -> None:
    assert not Path(".git").exists(), "existing Git repository must be preserved"
    existing = Path(".hg").exists()
    if existing:
        return
    write("init")
    write("branch", "--", arguments["base"])


def repository_present(arguments: dict[str, Any]) -> bool:
    assert not arguments
    assert not Path(".git").exists(), "existing Git repository must be preserved"
    return Path(".hg").is_dir()


def write(*arguments: str) -> None:
    environment = dict(os.environ, HGPLAIN="1")
    environment.pop("HGRCSKIPREPO", None)
    environment.pop("HGPLAINEXCEPT", None)
    result = subprocess.run(["hg", *arguments], env=environment, capture_output=True)
    assert result.returncode == 0, result.stderr.decode()


def configuration(key: str) -> str:
    missing = not Path(".hg").exists()
    if missing:
        return ""
    environment = dict(os.environ, HGPLAIN="1", HGRCPATH="")
    environment.pop("HGRCSKIPREPO", None)
    environment.pop("HGPLAINEXCEPT", None)
    result = subprocess.run(["hg", "config", key], env=environment, capture_output=True)
    assert result.returncode in (0, 1), result.stderr.decode()
    return result.stdout.decode().strip()


def registration(arguments: dict[str, Any]) -> list[tuple[str, str, str]]:
    assert not Path(".git").exists(), "existing Git repository must be preserved"
    directory = arguments["directory"]
    assert not any(char in directory for char in "\n\r\0"), "invalid hook directory"
    desired = {
        "hooks.pretxncommit.agentrig": "sh " + shlex.quote(directory + "/pretxncommit"),
        "ui.ignore.agentrig": directory + ".hgignore",
    }
    return [(key, configuration(key), value) for key, value in desired.items()]


def generate(arguments: dict[str, Any]) -> dict[str, Any]:
    directory = arguments["directory"]
    binary = arguments["binary"]
    hook = (
        "#!/bin/sh\nset -eu\nroot=$(hg root)\n"
        f'{binary} guard-commit --root "$root" --revision "$HG_NODE"\n'
        f'exec {binary} check --root "$root" --revision "$HG_NODE"\n'
    )
    ignored = "syntax: glob\n" + "".join(path + "/**\n" for path in arguments["ignored"])
    return {
        "root_command": "hg root",
        "files": {directory + "/pretxncommit": hook, directory + ".hgignore": ignored},
    }


def register_hooks(arguments: dict[str, Any]) -> None:
    values = registration(arguments)
    assert all(not current or current == desired for _, current, desired in values), (
        "setup conflict"
    )
    changes = []
    for key, current, desired in values:
        missing = current != desired
        if missing:
            section, name = key.split(".", 1)
            changes.append(f"\n[{section}]\n{name} = {desired}\n")
    unchanged = not changes
    if unchanged:
        return
    with Path(".hg/hgrc").open("a") as output:
        output.write("".join(changes))


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
    "initialize": initialize,
    "repository-present": repository_present,
    "registration": registration,
    "generate": generate,
    "register-hooks": register_hooks,
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
