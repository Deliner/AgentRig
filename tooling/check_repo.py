#!/usr/bin/env python3

from __future__ import annotations

import argparse
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path

from comment_parser import DECISION_ROW, INVARIANT_ROW, LINK, source_comments, table_rows
from plan_policy import plan_errors

# DECISION: D002
# DECISION: D008
# DECISION: D009
# DECISION: D012
# DECISION: D013
# DECISION: D016

LEDGER = Path("Ledger")
DECISIONS = LEDGER / "Decisions.md"
INVARIANTS = LEDGER / "Invariants.md"
PLAN = LEDGER / "Plan.md"
STATE = LEDGER / "State.md"


@dataclass(frozen=True)
class Finding:
    level: str
    message: str


def git(root: Path, *args: str) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(["git", *args], cwd=root, check=False, capture_output=True)


def history_file(history: Path, path: Path) -> subprocess.CompletedProcess[bytes]:
    result = git(history, "show", f"HEAD:{path}")
    if result.returncode == 0 or path.parts[:1] != ("Ledger",):
        return result
    return git(history, "show", f"HEAD:{Path('ledger', *path.parts[1:])}")


def repository_files(root: Path) -> list[Path]:
    result = git(root, "ls-files", "--cached", "--others", "--exclude-standard", "-z")
    if result.returncode != 0:
        return [
            path
            for path in root.rglob("*")
            if path.is_file() and "__pycache__" not in path.parts and path.suffix != ".pyc"
        ]
    return [root / item.decode() for item in result.stdout.split(b"\0") if item]


def inside(root: Path, base: Path, target: str) -> Path | None:
    try:
        path = (base / target).resolve()
        path.relative_to(root.resolve())
        return path
    except (OSError, RuntimeError, ValueError):
        return None


def decision_findings(root: Path, history: Path) -> list[Finding]:
    findings: list[Finding] = []
    rows = table_rows(root / DECISIONS, DECISION_ROW)
    ids = [row.group(1) for row in rows]
    if len(ids) != len(set(ids)):
        findings.append(Finding("error", "decision IDs are not unique"))
    for row in rows:
        decision_id, detail, _statement, applications = row.groups()
        if detail != f"Decisions/{decision_id[1:]}.md":
            findings.append(Finding("error", f"{decision_id}: detail path does not match ID"))
        detail_path = root / LEDGER / detail
        if not detail_path.is_file():
            findings.append(Finding("error", f"{decision_id}: missing detail file"))
        else:
            source = detail_path.read_text(encoding="utf-8")
            sections = re.findall(
                r"^## (\w+)\n\n(.+?)(?=\n## |\Z)", source, re.MULTILINE | re.DOTALL
            )
            required = ["Context", "Chosen", "Rejected", "Rationale", "Consequences"]
            if [name for name, body in sections if body.strip()] != required:
                findings.append(Finding("error", f"{decision_id}: incomplete choice tuple"))
        for target in LINK.findall(applications):
            path = inside(root, root / LEDGER, target)
            if path is None or not path.exists():
                findings.append(Finding("error", f"{decision_id}: missing application {target}"))
                continue
            comments = source_comments(path)
            if comments is not None and f"# DECISION: {decision_id}" not in {
                comment for _, comment in comments
            }:
                findings.append(Finding("error", f"{decision_id}: marker absent from {target}"))
    known = set(ids)
    for path in (root / LEDGER / "Decisions").glob("[0-9][0-9][0-9].md"):
        if f"D{path.stem}" not in known:
            findings.append(Finding("error", f"{path.relative_to(root)} is not indexed"))
    findings.extend(committed_decision_findings(root, history, rows))
    return findings


def committed_decision_findings(
    root: Path, history: Path, current_rows: list[re.Match[str]]
) -> list[Finding]:
    previous = history_file(history, DECISIONS)
    if previous.returncode != 0:
        return []
    old_rows = [
        match
        for line in previous.stdout.decode().splitlines()
        if (match := DECISION_ROW.match(line))
    ]
    current = {row.group(1): row for row in current_rows}
    findings: list[Finding] = []
    for old in old_rows:
        decision_id, detail, statement, applications = old.groups()
        new = current.get(decision_id)
        stable = new is not None and new.group(2) == detail and new.group(3) == statement
        delivered = new is not None and set(LINK.findall(applications)) <= set(
            LINK.findall(new.group(4))
        )
        old_detail = history_file(history, LEDGER / detail)
        current_detail = root / LEDGER / detail
        detail_stable = (
            old_detail.returncode != 0
            or current_detail.is_file()
            and old_detail.stdout == current_detail.read_bytes()
        )
        if not stable or not delivered or not detail_stable:
            findings.append(Finding("error", f"{decision_id}: committed record changed"))
    return findings


def invariant_findings(root: Path) -> list[Finding]:
    findings: list[Finding] = []
    rows = table_rows(root / INVARIANTS, INVARIANT_ROW)
    ids = [row.group(1) for row in rows]
    if len(ids) != len(set(ids)):
        findings.append(Finding("error", "invariant IDs are not unique"))
    for row in rows:
        invariant_id, detail, _statement, test_name, target = row.groups()
        if detail != f"Invariants/{invariant_id[1:]}.md":
            findings.append(Finding("error", f"{invariant_id}: detail path does not match ID"))
        detail_path = root / LEDGER / detail
        if not detail_path.is_file():
            findings.append(Finding("error", f"{invariant_id}: missing detail file"))
        else:
            source = detail_path.read_text(encoding="utf-8")
            sections = re.findall(
                r"^## ([A-Za-z ]+)\n\n(.+?)(?=\n## |\Z)",
                source,
                re.MULTILINE | re.DOTALL,
            )
            if [name for name, body in sections if body.strip()] != ["Predicate", "Oracle"]:
                findings.append(Finding("error", f"{invariant_id}: incomplete detail"))
        path = inside(root, root / LEDGER, target)
        if path is None or not path.is_file():
            findings.append(Finding("error", f"{invariant_id}: missing test {target}"))
            continue
        source = path.read_text(encoding="utf-8")
        marker = re.search(
            rf"^\s*# INVARIANT: {invariant_id}\n(?:async )?def {test_name}\(",
            source,
            re.MULTILINE,
        )
        if marker is None:
            findings.append(Finding("error", f"{invariant_id}: marked test is absent"))
    known = set(ids)
    for path in (root / LEDGER / "Invariants").glob("[0-9][0-9][0-9].md"):
        if f"I{path.stem}" not in known:
            findings.append(Finding("error", f"{path.relative_to(root)} is not indexed"))
    return findings


def plan_findings(root: Path) -> list[Finding]:
    return [Finding("error", message) for message in plan_errors(root)]


def state_findings(root: Path) -> list[Finding]:
    path = root / STATE
    if not path.is_file():
        return [Finding("error", "missing required file Ledger/State.md")]
    parts = re.split(r"^## ([^\n]+)\n", path.read_text(encoding="utf-8"), flags=re.MULTILINE)
    required = ["Focus", "Workspace", "Progress", "Verification", "Blockers", "Next action"]
    if parts[1::2] != required or not all(body.strip() for body in parts[2::2]):
        return [Finding("error", "State requires nonempty recovery sections in canonical order")]
    return []


def check(root: Path, history: Path) -> list[Finding]:
    required = [
        root / "Project" / "README.md",
        root / DECISIONS,
        root / INVARIANTS,
        root / PLAN,
    ]
    missing = [path for path in required if not path.is_file()]
    if missing:
        return [
            Finding("error", f"missing required file {path.relative_to(root)}") for path in missing
        ]
    findings = decision_findings(root, history)
    findings.extend(invariant_findings(root))
    findings.extend(plan_findings(root))
    findings.extend(state_findings(root))
    return findings


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--history-root", type=Path)
    args = parser.parse_args()
    root = args.root.resolve()
    history = (args.history_root or root).resolve()
    try:
        findings = check(root, history)
    except (OSError, UnicodeError, subprocess.CalledProcessError) as error:
        findings = [Finding("error", str(error))]
    for finding in findings:
        print(f"{finding.level.upper()}: {finding.message}")
    return int(any(finding.level == "error" for finding in findings))


if __name__ == "__main__":
    raise SystemExit(main())
