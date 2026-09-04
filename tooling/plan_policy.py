from __future__ import annotations

import re
from dataclasses import dataclass
from graphlib import CycleError, TopologicalSorter
from pathlib import Path

# DECISION: D012

HEADER = "| ID | Status | Depends on | Feature | User capability |"
SEPARATOR = "| --- | --- | --- | --- | --- |"
ROW = re.compile(
    r"^\| \[(P\d{3})\]\((Plan/\d{3}\.md)\) "
    r"\| (pending|active|paused|complete) \| (-|P\d{3}(?:, P\d{3})*) "
    r"\| ([^|]+) \| ([^|]+) \|$"
)


@dataclass(frozen=True)
class Feature:
    identity: str
    detail: str
    status: str
    dependencies: tuple[str, ...]


def read_features(root: Path) -> tuple[list[Feature], list[str]]:
    lines = (root / "Ledger/Plan.md").read_text(encoding="utf-8").splitlines()
    errors: list[str] = []
    features: list[Feature] = []
    if lines.count(HEADER) != 1 or lines.count(SEPARATOR) != 1:
        errors.append("Plan requires one canonical feature table header and separator")
    for number, line in enumerate(lines, 1):
        if not line.lstrip().startswith("|") or line in {HEADER, SEPARATOR}:
            continue
        match = ROW.fullmatch(line)
        if match is None:
            errors.append(f"Plan line {number}: malformed feature row")
            continue
        identity, detail, status, dependencies, title, capability = match.groups()
        if not title.strip() or not capability.strip():
            errors.append(f"{identity}: feature and user capability must be nonempty")
        features.append(
            Feature(
                identity,
                detail,
                status,
                () if dependencies == "-" else tuple(dependencies.split(", ")),
            )
        )
    return features, errors


def detail_errors(root: Path, feature: Feature) -> list[str]:
    identity = feature.identity
    if feature.detail != f"Plan/{identity[1:]}.md":
        return [f"{identity}: detail path does not match ID"]
    path = root / "Ledger" / feature.detail
    if not path.is_file():
        return [f"{identity}: missing detail file"]
    source = path.read_text(encoding="utf-8")
    parts = re.split(r"^## ([^\n]+)\n", source, flags=re.MULTILINE)
    names, bodies = parts[1::2], parts[2::2]
    required = ["Feature", "User capability", "Acceptance"]
    if names not in (required, [*required, "Delivery"]) or not all(body.strip() for body in bodies):
        return [f"{identity}: incomplete feature contract or invalid detail sections"]
    if feature.status in {"paused", "complete"} and "Delivery" not in names:
        return [f"{identity}: {feature.status} feature requires Delivery context"]
    return []


def plan_errors(root: Path) -> list[str]:
    features, errors = read_features(root)
    indexed = {feature.identity: feature for feature in features}
    if len(indexed) != len(features):
        errors.append("Plan IDs are not unique")
    if sum(feature.status == "active" for feature in features) > 1:
        errors.append("Plan may contain at most one active feature")
    for feature in features:
        errors.extend(detail_errors(root, feature))
        if len(set(feature.dependencies)) != len(feature.dependencies):
            errors.append(f"{feature.identity}: duplicate dependencies")
        for dependency in feature.dependencies:
            if dependency not in indexed:
                errors.append(f"{feature.identity}: unknown dependency {dependency}")
            elif feature.status in {"active", "complete"} and (
                indexed[dependency].status != "complete"
            ):
                errors.append(
                    f"{feature.identity}: {feature.status} feature requires "
                    f"completed dependency {dependency}"
                )
    graph = {feature.identity: feature.dependencies for feature in features}
    try:
        TopologicalSorter(graph).prepare()
    except CycleError:
        errors.append("Plan dependencies contain a cycle")
    for path in (root / "Ledger/Plan").glob("[0-9][0-9][0-9].md"):
        if f"P{path.stem}" not in indexed:
            errors.append(f"{path.relative_to(root)} is not indexed")
    return errors
