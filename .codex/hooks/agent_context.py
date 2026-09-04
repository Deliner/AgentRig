#!/usr/bin/env python3

"""Shared session and pre-tool entrypoint for worker context and edit guidance."""

from __future__ import annotations

import json
import re
import shlex
import sys
from pathlib import Path
from typing import Any

import complexity_discipline_reminder as complexity
from just_command_guard import command_from, rejection

# DECISION: D013
# DECISION: D014

ROOT = Path(__file__).resolve().parents[2]
LEDGER_SKILLS = {
    "Plan": "edit-plan",
    "Decisions": "edit-decisions",
    "Invariants": "edit-invariants",
    "State": "edit-state",
}
EDIT_TOOLS = {"apply_patch", "Edit", "Write"}
SHELL_TOOLS = {"Bash", "Shell", "exec_command"}
PATCH_PATH = re.compile(
    r"^\*\*\* (?:Add File|Update File|Delete File|Move to): (.+)$", re.MULTILINE
)


def context(event_name: str, message: str) -> dict[str, Any]:
    return {"hookSpecificOutput": {"hookEventName": event_name, "additionalContext": message}}


def deny(reason: str) -> dict[str, Any]:
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    }


def edit_paths(event: dict[str, Any]) -> list[str]:
    payload = event.get("tool_input")
    if isinstance(payload, str):
        return PATCH_PATH.findall(payload)
    if not isinstance(payload, dict):
        return []
    paths = [payload[key] for key in ("file_path", "path") if isinstance(payload.get(key), str)]
    for key in ("patch", "input", "patch_text"):
        value = payload.get(key)
        if isinstance(value, str):
            paths.extend(PATCH_PATH.findall(value))
    return paths


def affected_skills(event: dict[str, Any], root: Path) -> list[str]:
    cwd = Path(str(event.get("cwd") or root))
    if not cwd.is_absolute():
        cwd = root / cwd
    names: set[str] = set()
    for value in edit_paths(event):
        try:
            relative = (cwd / value).resolve().relative_to(root.resolve())
        except (OSError, RuntimeError, ValueError):
            continue
        for ledger, skill in LEDGER_SKILLS.items():
            if relative == Path("Ledger", f"{ledger}.md") or (
                ledger != "State"
                and relative.is_relative_to(Path("Ledger", ledger))
                and relative.suffix == ".md"
            ):
                names.add(skill)
    return [skill for skill in LEDGER_SKILLS.values() if skill in names]


def guidance(skills: list[str], root: Path) -> str:
    return "\n".join(
        f"Before editing the corresponding Ledger file, read and apply {root / '.agents/skills' / skill / 'SKILL.md'}."
        for skill in skills
    )


def reminder(event: dict[str, Any], root: Path) -> dict[str, Any] | None:
    schedule = root / ".agents/skills/complexity-discipline/context-reminder.json"
    return complexity.pre_tool_use(event, *complexity.configuration(schedule))


def dispatch(event: dict[str, Any], root: Path = ROOT) -> dict[str, Any] | None:
    event_name = event.get("hook_event_name")
    if event_name == "SessionStart":
        message = (
            f"Resume from {root / 'Ledger/State.md'} and {root / 'Ledger/Plan.md'}. "
            "Compare the recorded task, VAC, checks, blockers, and next action with current "
            "Git status, diff, and recent commits before acting. State may be stale after an "
            "interruption; current contracts and Git take precedence. Missing State is a "
            "recovery task, not evidence that previous work completed.\n\n"
            + complexity.DEFAULT_FULL_MESSAGE
        )
        try:
            return complexity.session_start(event, message)
        except OSError as error:
            print(f"session reminder state: {error}", file=sys.stderr)
            return context("SessionStart", message)
    if event_name != "PreToolUse":
        return None
    tool = event.get("tool_name")
    if tool in SHELL_TOOLS:
        command = command_from(event)
        reason = rejection(command, root)
        if reason:
            return deny(reason)
        argv = shlex.split(command or "")
        if len(argv) > 1 and argv[1] == "write":
            return context(
                "PreToolUse",
                "Shell write targets are opaque. If this command changes Ledger, apply only "
                "the matching editing skill before the write:\n"
                + guidance(list(LEDGER_SKILLS.values()), root),
            )
        return None
    if tool not in EDIT_TOOLS:
        return None
    message = guidance(affected_skills(event, root), root)
    try:
        result = reminder(event, root)
    except (OSError, ValueError) as error:
        print(f"complexity reminder: {error}", file=sys.stderr)
        result = None
    if result is not None:
        if message:
            output = result["hookSpecificOutput"]
            output["permissionDecisionReason"] += "\n\n" + message
        return result
    return context("PreToolUse", message) if message else None


def main() -> int:
    try:
        event = json.load(sys.stdin)
        if not isinstance(event, dict):
            raise ValueError("expected a hook event object")
        result = dispatch(event)
    except (OSError, ValueError, TypeError) as error:
        result = deny(f"Worker hook could not process this event: {error}")
    if result is not None:
        print(json.dumps(result, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
