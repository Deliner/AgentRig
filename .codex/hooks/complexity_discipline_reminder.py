#!/usr/bin/env python3

"""Project-local Codex adapter for the complexity-discipline reminder hook."""

# Historical Python parity reference; production execution uses tooling/worker (D015/D016).
from __future__ import annotations

import argparse
import contextlib
import fcntl
import hashlib
import json
import os
import sys
import tempfile
from collections.abc import Iterator
from pathlib import Path
from typing import Any

DEFAULT_ATTENTION_INTERVAL = 35_000
DEFAULT_FULL_INTERVAL = 140_000
DEFAULT_ATTENTION_MESSAGE = (
    "COMPLEXITY_DISCIPLINE_CHECKPOINT\n\n"
    "Reassess the current approach before making the next implementation decision.\n\n"
    "The purpose of this checkpoint is to preserve semantic alignment with the current "
    "requirements. Every distinction, mechanism, validation step, lifecycle rule, and "
    "guarantee should correspond either to an explicit requirement or to a necessary "
    "condition for correctness.\n\n"
    "Evidence of alignment: removing an element would violate a stated requirement or make "
    "the implementation incorrect.\n\n"
    "Boundary of the check: do not reinterpret or strengthen the requirements, expand the "
    "scope, or introduce new process merely to make the solution appear more complete. If no "
    "unsupported complexity is present, continue without redesigning the approach."
)
DEFAULT_FULL_MESSAGE = (
    "COMPLEXITY_DISCIPLINE_FULL_REFRESH_REQUIRED\n\n"
    "Before any further reasoning or action, apply the complete skill at:\n\n"
    ".agents/skills/complexity-discipline/SKILL.md\n\n"
    "Resolve this path relative to the repository root, regardless of the current working "
    "directory.\n\n"
    "Load and apply the complete current instructions from the specified skill source using "
    "a read-only mechanism permitted by the active environment, then reassess the deferred "
    "approach before retrying it.\n\n"
    "Evidence of completion: the complete current skill has been applied during this refresh "
    "and the active approach has been reassessed against all of its instructions.\n\n"
    "Boundary of completion: prior knowledge, an earlier application, a summary, metadata "
    "inspection, keyword search, or applying only part of the skill does not satisfy this "
    "requirement.\n\n"
    "If the skill cannot be loaded or applied completely, stop without continuing the "
    "original task and report that the mandatory refresh could not be performed."
)

# DECISION: D006


def load_object(path: Path) -> dict[str, Any]:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"expected a JSON object: {path}")
    return value


def configuration(path: Path) -> tuple[int, int, str, str]:
    try:
        value = load_object(path)
    except (OSError, ValueError, json.JSONDecodeError):
        value = {}
    attention = value.get("attention_interval_tokens", DEFAULT_ATTENTION_INTERVAL)
    full = value.get("full_refresh_interval_tokens", DEFAULT_FULL_INTERVAL)
    message = value.get("attention_message", DEFAULT_ATTENTION_MESSAGE)
    attention = (
        attention
        if isinstance(attention, int) and not isinstance(attention, bool) and attention > 0
        else DEFAULT_ATTENTION_INTERVAL
    )
    full = (
        full
        if isinstance(full, int) and not isinstance(full, bool) and full > attention
        else DEFAULT_FULL_INTERVAL
    )
    if full <= attention:
        attention, full = DEFAULT_ATTENTION_INTERVAL, DEFAULT_FULL_INTERVAL
    if not isinstance(message, str) or not message.strip():
        message = DEFAULT_ATTENTION_MESSAGE
    return attention, full, message, DEFAULT_FULL_MESSAGE


def safe_key(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def state_root(event: dict[str, Any]) -> Path:
    override = os.environ.get("COMPLEXITY_DISCIPLINE_STATE_DIR")
    if override:
        return Path(override)
    workspace = str(event.get("cwd") or Path.cwd())
    return (
        Path(os.environ.get("XDG_STATE_HOME", Path.home() / ".local" / "state"))
        / "codex"
        / "complexity-discipline"
        / safe_key(workspace)
    )


@contextlib.contextmanager
def locked_state(event: dict[str, Any]) -> Iterator[tuple[Path, dict[str, Any]]]:
    session = str(event.get("session_id") or event.get("transcript_path") or "session")
    root = state_root(event)
    root.mkdir(parents=True, exist_ok=True)
    stem = safe_key(session)
    lock_path = root / f"{stem}.lock"
    state_path = root / f"{stem}.json"
    with lock_path.open("a", encoding="utf-8") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        try:
            state = load_object(state_path) if state_path.exists() else {}
        except (OSError, ValueError, json.JSONDecodeError):
            state = {}
        yield state_path, state


def save_state(path: Path, state: dict[str, Any]) -> None:
    descriptor, temporary = tempfile.mkstemp(prefix=f".{path.name}-", dir=path.parent)
    temporary_path = Path(temporary)
    try:
        with os.fdopen(descriptor, "w", encoding="utf-8") as output:
            json.dump(state, output, separators=(",", ":"))
            output.write("\n")
        os.replace(temporary_path, path)
    finally:
        temporary_path.unlink(missing_ok=True)


def state_integer(value: Any, fallback: int, minimum: int = 0) -> int:
    return (
        value
        if isinstance(value, int) and not isinstance(value, bool) and value >= minimum
        else fallback
    )


def transcript_tokens(path: Path, offset: int) -> tuple[list[int], int]:
    size = path.stat().st_size
    if offset < 0 or offset > size:
        offset = 0
    tokens: list[int] = []
    next_offset = offset
    with path.open("rb") as transcript:
        transcript.seek(offset)
        while line := transcript.readline():
            try:
                record = json.loads(line)
            except json.JSONDecodeError:
                if not line.endswith(b"\n"):
                    break
                next_offset = transcript.tell()
                continue
            next_offset = transcript.tell()
            if not isinstance(record, dict):
                continue
            if record.get("type") != "event_msg":
                continue
            payload = record.get("payload")
            if not isinstance(payload, dict) or payload.get("type") != "token_count":
                continue
            info = payload.get("info")
            usage = info.get("last_token_usage") if isinstance(info, dict) else None
            value = usage.get("input_tokens") if isinstance(usage, dict) else None
            if isinstance(value, int) and not isinstance(value, bool) and value >= 0:
                tokens.append(value)
    return tokens, next_offset


def session_start(event: dict[str, Any], message: str) -> dict[str, Any]:
    transcript = event.get("transcript_path")
    scan_offset = 0
    baseline = 0
    if isinstance(transcript, str) and transcript:
        with contextlib.suppress(OSError):
            tokens, scan_offset = transcript_tokens(Path(transcript), 0)
            baseline = tokens[-1] if tokens else 0
    with locked_state(event) as (path, state):
        state.clear()
        state.update(
            {
                "transcript_path": transcript or "",
                "scan_offset": scan_offset,
                "baseline_tokens": baseline,
                "latest_tokens": baseline,
                "allow_next_edit": False,
            }
        )
        save_state(path, state)
    return {
        "hookSpecificOutput": {
            "hookEventName": "SessionStart",
            "additionalContext": message,
        }
    }


def pre_tool_use(
    event: dict[str, Any],
    attention: int,
    full: int,
    attention_message: str,
    full_message: str,
) -> dict[str, Any] | None:
    transcript_value = event.get("transcript_path")
    if not isinstance(transcript_value, str) or not transcript_value:
        return None
    transcript = Path(transcript_value)
    with locked_state(event) as (path, state):
        if state.get("transcript_path") != transcript_value:
            state = {
                "transcript_path": transcript_value,
                "scan_offset": 0,
                "baseline_tokens": 0,
                "allow_next_edit": False,
                "next_attention_tokens": attention,
            }
        state["scan_offset"] = state_integer(state.get("scan_offset"), 0)
        state["baseline_tokens"] = state_integer(state.get("baseline_tokens"), 0)
        next_attention = state_integer(state.get("next_attention_tokens"), attention, 1)
        last_attention = ((full + attention - 1) // attention) * attention
        state["next_attention_tokens"] = (
            next_attention
            if next_attention % attention == 0 and next_attention <= last_attention
            else attention
        )
        state["allow_next_edit"] = state.get("allow_next_edit") is True
        latest_value = state.get("latest_tokens")
        if not isinstance(latest_value, int) or isinstance(latest_value, bool) or latest_value < 0:
            state.pop("latest_tokens", None)
        tokens, next_offset = transcript_tokens(transcript, state["scan_offset"])
        latest = state.get("latest_tokens")
        for current in tokens:
            if isinstance(latest, int) and current < latest:
                state["baseline_tokens"] = current
                state["allow_next_edit"] = False
                state["next_attention_tokens"] = attention
            latest = current
        state["scan_offset"] = next_offset
        if isinstance(latest, int):
            state["latest_tokens"] = latest
            if state["baseline_tokens"] > latest:
                state["baseline_tokens"] = latest
                state["next_attention_tokens"] = attention
                state["allow_next_edit"] = False

        reminder: str | None = None
        if isinstance(latest, int):
            if state.get("allow_next_edit"):
                state["allow_next_edit"] = False
            else:
                delta = latest - int(state.get("baseline_tokens", 0))
                next_attention = int(state.get("next_attention_tokens", attention))
                if delta >= full:
                    state["baseline_tokens"] = latest
                    state["next_attention_tokens"] = attention
                    state["allow_next_edit"] = True
                    reminder = full_message
                elif delta >= next_attention:
                    state["next_attention_tokens"] = (delta // attention + 1) * attention
                    state["allow_next_edit"] = True
                    reminder = attention_message
        save_state(path, state)

    if reminder is None:
        return None
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reminder,
        }
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", required=True, type=Path)
    args = parser.parse_args()
    try:
        event = json.load(sys.stdin)
        if not isinstance(event, dict):
            return 0
        attention, full, attention_message, full_message = configuration(args.config)
        name = event.get("hook_event_name")
        result = session_start(event, full_message) if name == "SessionStart" else None
        if name == "PreToolUse":
            result = pre_tool_use(event, attention, full, attention_message, full_message)
        if result is not None:
            json.dump(result, sys.stdout, ensure_ascii=False)
            sys.stdout.write("\n")
    except Exception as error:
        print(f"complexity-discipline hook: {error}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
