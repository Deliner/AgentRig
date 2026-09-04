from __future__ import annotations

from collections.abc import Iterable
from pathlib import Path

SOURCE_SUFFIXES = {".py", ".sh"}
TEXT_SUFFIXES = SOURCE_SUFFIXES | {".json", ".md", ".toml", ".txt", ".yaml", ".yml"}


def source_action(path: Path, level: str) -> str:
    if path.suffix not in SOURCE_SUFFIXES:
        return ""
    if level == "error":
        return (
            " ACTION: Apply $refactor-large-file. Extract a cohesive responsibility while "
            "preserving observable behavior and interfaces; do not pass the limit through "
            "compression or arbitrary splitting."
        )
    return (
        " ACTION: Consider $refactor-large-file before adding another responsibility to this file."
    )


def directory_action(level: str) -> str:
    if level == "error":
        return (
            " ACTION: Apply $refactor-large-directory. Group cohesive responsibilities while "
            "preserving paths, dependencies, and observable behavior; do not create arbitrary "
            "directories."
        )
    return " ACTION: Consider $refactor-large-directory before adding another responsibility."


# DECISION: D007
def size_findings(root: Path, paths: Iterable[Path]) -> list[tuple[str, str]]:
    findings: list[tuple[str, str]] = []
    children: dict[Path, set[str]] = {}
    for path in paths:
        if not path.exists():
            continue
        relative = path.relative_to(root)
        parts = relative.parts
        for index, name in enumerate(parts):
            parent = Path(*parts[:index])
            children.setdefault(parent, set()).add(name)
        if path.suffix not in TEXT_SUFFIXES:
            continue
        try:
            meaningful = sum(
                bool(line.strip()) for line in path.read_text(encoding="utf-8").splitlines()
            )
        except UnicodeDecodeError:
            continue
        if meaningful > 500:
            level, limit = "error", 500
        elif meaningful > 300:
            level, limit = "warning", 300
        else:
            continue
        message = (
            f"{relative}: {meaningful} meaningful lines exceeds {limit}."
            f"{source_action(path, level)}"
        )
        findings.append((level, message))
    for directory, names in children.items():
        label = str(directory) if directory.parts else "."
        if directory == Path("ledger/Decisions"):
            continue
        if len(names) > 15:
            level, limit = "error", 15
        elif len(names) > 10:
            level, limit = "warning", 10
        else:
            continue
        findings.append(
            (
                level,
                f"{label}: {len(names)} items exceeds {limit}.{directory_action(level)}",
            )
        )
    return findings
