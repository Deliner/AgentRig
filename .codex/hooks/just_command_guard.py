# Historical Python parity reference; production execution uses tooling/worker (D015/D016).
from __future__ import annotations

import json
import shlex
import sys
from pathlib import Path
from typing import Any

# DECISION: D005

ROOT = Path(__file__).resolve().parents[2]


def outer_shell_control(command: str) -> bool:
    quote = ""
    escaped = False
    index = 0
    while index < len(command):
        char = command[index]
        if escaped:
            escaped = False
        elif char == "\\" and quote != "'":
            escaped = True
        elif quote:
            if char == quote:
                quote = ""
            elif quote == '"' and (char == "`" or command[index : index + 2] == "$("):
                return True
        elif char in {"'", '"'}:
            quote = char
        elif char in ";|&<>\n`" or command[index : index + 2] == "$(":
            return True
        index += 1
    return bool(quote or escaped)


def command_from(payload: dict[str, Any]) -> str | None:
    tool_input = payload.get("tool_input")
    if not isinstance(tool_input, dict):
        return None
    for key in ("cmd", "command"):
        value = tool_input.get(key)
        if isinstance(value, str):
            return value
    return None


def rejection(command: str | None, root: Path = ROOT) -> str | None:
    if command is None or outer_shell_control(command):
        return "Agent shell commands must be one top-level `just` invocation."
    try:
        argv = shlex.split(command)
    except ValueError:
        return "Agent shell command could not be parsed; use one `just` invocation."
    if not argv or Path(argv[0]).name != "just":
        return "Direct shell commands are disabled; use a recipe from `just list`."
    catalog = json.loads((root / "tooling" / "command_catalog.json").read_text(encoding="utf-8"))
    if len(argv) == 1:
        return None
    if argv[1].startswith("-") or argv[1] not in catalog:
        return f"Unknown or bypassing Just invocation {argv[1]!r}; use `just list`."
    return None


def main() -> int:
    try:
        payload = json.load(sys.stdin)
        reason = rejection(command_from(payload))
    except (json.JSONDecodeError, OSError, TypeError):
        reason = "Command guard could not validate this shell request."
    if reason is not None:
        print(
            json.dumps(
                {
                    "hookSpecificOutput": {
                        "hookEventName": "PreToolUse",
                        "permissionDecision": "deny",
                        "permissionDecisionReason": reason,
                    }
                }
            )
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
