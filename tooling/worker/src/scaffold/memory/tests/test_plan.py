from __future__ import annotations

from pathlib import Path

import pytest

from tooling.worker.src.scaffold.testing.consumer import CONFIG, invoke, memory, project

# DECISION: D012

Rows = list[tuple[int, str, str]]


def write_plan(root: Path, rows: Rows) -> None:
    uninitialized = not (root / "agentrig.yaml").exists()
    if uninitialized:
        project(root, CONFIG.replace('memory: "notes"', 'memory: "Ledger"'))
        memory(root).rename(root / "Ledger")
    details = root / "Ledger/Plan"
    details.mkdir(parents=True, exist_ok=True)
    table = [
        "# Plan",
        "",
        "| ID | Status | Depends on | Feature | User capability |",
        "| --- | --- | --- | --- | --- |",
    ]
    for identity, status, dependencies in rows:
        table.append(
            f"| [P{identity:03}](Plan/{identity:03}.md) | {status} | {dependencies} "
            "| Product outcome | Observable capability |"
        )
        delivery = ""
        paused = status == "paused"
        complete = status == "complete"
        if paused:
            delivery = (
                "\n## Delivery\n\nBlocked by a missing prior outcome. "
                "Retained branch: feature/example. Resume after prerequisites are delivered.\n"
            )
        elif complete:
            delivery = "\n## Delivery\n\nAcceptance verified by the feature test; passed.\n"
        (details / f"{identity:03}.md").write_text(
            f"# P{identity:03}\n\n## Feature\n\nProduct result.\n\n"
            "## User capability\n\nUser capability.\n\n"
            f"## Acceptance\n\nObservable acceptance.\n{delivery}",
            encoding="utf-8",
        )
    (root / "Ledger/Plan.md").write_text("\n".join(table) + "\n", encoding="utf-8")


@pytest.mark.parametrize(
    ("rows", "expected"),
    [
        ([], None),
        ([(1, "pending", "-")], None),
        ([(1, "active", "-")], None),
        ([(1, "paused", "-")], None),
        ([(1, "complete", "-")], None),
        ([(1, "complete", "-"), (2, "active", "P001")], None),
        ([(2, "active", "-"), (1, "paused", "P002")], None),
        ([(1, "active", "-"), (2, "pending", "P001")], None),
        ([(1, "active", "-"), (2, "active", "-")], "multiple active"),
        ([(1, "pending", "-"), (1, "pending", "-")], "invalid or duplicate ID"),
        ([(1, "pending", "P999")], "unknown dependency"),
        ([(1, "pending", "P001")], "cycle"),
        ([(1, "paused", "P002"), (2, "pending", "P001")], "cycle"),
        ([(1, "pending", "-"), (2, "active", "P001")], "prerequisite"),
        ([(1, "paused", "-"), (2, "complete", "P001")], "prerequisite"),
        ([(1, "complete", "-"), (2, "pending", "P001, P001")], "duplicate dependencies"),
        ([(1, "unknown", "-")], "invalid"),
        ([(1, "pending", "P1")], "unknown dependency"),
    ],
)
# INVARIANT: I009
def test_plan_delivery_contract(
    worker: Path, tmp_path: Path, rows: Rows, expected: str | None
) -> None:
    write_plan(tmp_path, rows)
    result = invoke(worker, tmp_path, "memory-check")
    valid = expected is None
    if valid:
        assert result.returncode == 0, result.stderr
    else:
        assert result.returncode == 2
        assert expected is not None and expected in result.stderr


@pytest.mark.parametrize(
    ("old", "new"),
    [
        ("## Feature", "## Implementation"),
        ("## Acceptance\n\nObservable acceptance.", "## Acceptance\n\n"),
        ("## User capability", "## Feature"),
        ("## Acceptance", "## Arbitrary\n\nUnexpected.\n\n## Acceptance"),
    ],
)
def test_invalid_detail_sections_fail(worker: Path, tmp_path: Path, old: str, new: str) -> None:
    write_plan(tmp_path, [(1, "pending", "-")])
    path = tmp_path / "Ledger/Plan/001.md"
    path.write_text(path.read_text(encoding="utf-8").replace(old, new), encoding="utf-8")
    assert invoke(worker, tmp_path, "memory-check").returncode == 2


@pytest.mark.parametrize("status", ["paused", "complete"])
def test_delivery_context_required(worker: Path, tmp_path: Path, status: str) -> None:
    write_plan(tmp_path, [(1, status, "-")])
    path = tmp_path / "Ledger/Plan/001.md"
    content = path.read_text(encoding="utf-8").split("\n## Delivery")[0]
    path.write_text(content, encoding="utf-8")
    assert "requires Delivery" in invoke(worker, tmp_path, "memory-check").stderr
    path.write_text(content + "\n## Delivery\n\n", encoding="utf-8")
    assert invoke(worker, tmp_path, "memory-check").returncode == 2


@pytest.mark.parametrize(
    "mutation",
    [
        ("(Plan/001.md)", "(Plan/002.md)", "detail path"),
        (" | Product outcome |", " |   |", "empty table cell"),
        (" | Observable capability |", " |   |", "empty table cell"),
        (" | pending | - |", " | pending |", "expected 5 columns"),
        ("Depends on", "Dependencies", "table header"),
    ],
)
def test_malformed_rows_are_not_silently_skipped(
    worker: Path, tmp_path: Path, mutation: tuple[str, str, str]
) -> None:
    old, new, expected = mutation
    write_plan(tmp_path, [(1, "pending", "-")])
    path = tmp_path / "Ledger/Plan.md"
    path.write_text(path.read_text(encoding="utf-8").replace(old, new), encoding="utf-8")
    assert expected in invoke(worker, tmp_path, "memory-check").stderr


def test_missing_and_unindexed_details_fail(worker: Path, tmp_path: Path) -> None:
    write_plan(tmp_path, [(1, "active", "-")])
    detail = tmp_path / "Ledger/Plan/001.md"
    detail.rename(detail.with_name("002.md"))
    result = invoke(worker, tmp_path, "memory-check")
    assert result.returncode == 2
    assert "missing or escaping" in result.stderr
    detail.write_text(detail.with_name("002.md").read_text())
    assert "unindexed detail" in invoke(worker, tmp_path, "memory-check").stderr


def test_blocker_followup_and_resume_lifecycle(worker: Path, tmp_path: Path) -> None:
    states: list[Rows] = [
        [(1, "active", "-")],
        [(1, "active", "-"), (2, "pending", "P001")],
        [(3, "active", "-"), (1, "paused", "P003"), (2, "pending", "P001")],
        [(3, "complete", "-"), (1, "paused", "P003"), (2, "pending", "P001")],
        [(3, "complete", "-"), (1, "active", "P003"), (2, "pending", "P001")],
        [(3, "complete", "-"), (1, "complete", "P003"), (2, "active", "P001")],
        [(3, "complete", "-"), (1, "complete", "P003"), (2, "complete", "P001")],
    ]
    for rows in states:
        write_plan(tmp_path, rows)
        assert invoke(worker, tmp_path, "memory-check").returncode == 0
