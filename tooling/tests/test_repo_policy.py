from __future__ import annotations

import subprocess
from pathlib import Path

from check_repo import check, decision_findings, invariant_findings, plan_findings
from size_policy import size_findings

# DECISION: D002
# DECISION: D007
# DECISION: D008
# DECISION: D009

ROOT = Path(__file__).parents[2]


def decision_detail(path: Path) -> None:
    path.write_text(
        "# D001\n\n"
        "## Context\n\nContext.\n\n"
        "## Chosen\n\nChosen.\n\n"
        "## Rejected\n\nRejected.\n\n"
        "## Rationale\n\nRationale.\n\n"
        "## Consequences\n\nConsequences.\n",
        encoding="utf-8",
    )


def minimal_ledger(root: Path, application: str = "../module.py") -> None:
    decisions = root / "Ledger" / "Decisions"
    decisions.mkdir(parents=True)
    (root / "Ledger" / "Decisions.md").write_text(
        "# Decisions\n\n"
        "| ID | Decision | Applies in |\n"
        "| --- | --- | --- |\n"
        f"| [D001](Decisions/001.md) | Choice. | [code]({application}) |\n",
        encoding="utf-8",
    )
    decision_detail(decisions / "001.md")
    (root / "Ledger" / "Invariants.md").write_text(
        "# Invariants\n\n| ID | Invariant | Enforced by |\n| --- | --- | --- |\n",
        encoding="utf-8",
    )


# INVARIANT: I001
def test_decision_applications_require_markers(tmp_path: Path) -> None:
    minimal_ledger(tmp_path)
    source = tmp_path / "module.py"
    source.write_text("value = 1\n", encoding="utf-8")
    assert any("marker absent" in item.message for item in decision_findings(tmp_path, tmp_path))
    source.write_text("# DECISION: D001\nvalue = 1\n", encoding="utf-8")
    assert decision_findings(tmp_path, tmp_path) == []


# INVARIANT: I002
def test_invariant_links_require_marked_tests(tmp_path: Path) -> None:
    minimal_ledger(tmp_path)
    tests = tmp_path / "tests"
    tests.mkdir()
    target = tests / "test_policy.py"
    target.write_text("def test_rule():\n    pass\n", encoding="utf-8")
    details = tmp_path / "Ledger" / "Invariants"
    details.mkdir()
    (details / "001.md").write_text(
        "# I001\n\n## Predicate\n\nRule.\n\n## Oracle\n\nOracle.\n",
        encoding="utf-8",
    )
    (tmp_path / "Ledger" / "Invariants.md").write_text(
        "# Invariants\n\n"
        "| ID | Invariant | Enforced by |\n"
        "| --- | --- | --- |\n"
        "| [I001](Invariants/001.md) | Rule. | [test_rule](../tests/test_policy.py) |\n",
        encoding="utf-8",
    )
    assert invariant_findings(tmp_path)
    target.write_text("# INVARIANT: I001\ndef test_rule():\n    pass\n", encoding="utf-8")
    assert invariant_findings(tmp_path) == []


# INVARIANT: I003
def test_committed_decisions_are_append_only(tmp_path: Path) -> None:
    minimal_ledger(tmp_path)
    (tmp_path / "module.py").write_text("# DECISION: D001\nvalue = 1\n", encoding="utf-8")
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    subprocess.run(["git", "add", "."], cwd=tmp_path, check=True)
    subprocess.run(
        [
            "git",
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-qm",
            "base",
        ],
        cwd=tmp_path,
        check=True,
    )
    index = tmp_path / "Ledger" / "Decisions.md"
    index.write_text(
        index.read_text(encoding="utf-8").replace("Choice.", "Changed."), encoding="utf-8"
    )
    assert any(
        "committed record changed" in item.message for item in decision_findings(tmp_path, tmp_path)
    )


# INVARIANT: I004
def test_size_thresholds_warn_then_fail(tmp_path: Path) -> None:
    source = tmp_path / "module.py"
    source.write_text("value = 1\n" * 301, encoding="utf-8")
    assert size_findings(tmp_path, [source])[0][0] == "warning"
    source.write_text("value = 1\n" * 501, encoding="utf-8")
    assert size_findings(tmp_path, [source])[0][0] == "error"


def test_worker_repository_policy_passes() -> None:
    assert [item for item in check(ROOT, ROOT) if item.level == "error"] == []


# INVARIANT: I008
def test_worker_layout_and_detail_indexes() -> None:
    assert (ROOT / "Project" / "README.md").is_file()
    assert (ROOT / "Ledger" / "Plan").is_dir()
    assert (ROOT / "Ledger" / "Decisions").is_dir()
    assert (ROOT / "Ledger" / "Invariants").is_dir()
    assert invariant_findings(ROOT) == []
    assert decision_findings(ROOT, ROOT) == []


# INVARIANT: I009
def test_plan_has_one_active_feature_with_complete_detail() -> None:
    assert plan_findings(ROOT) == []
