# DECISION: D020
# DECISION: D012
# DECISION: D009
# DECISION: D008
# DECISION: D007
# DECISION: D002
import json
import subprocess
from pathlib import Path

import pytest
from support import invoke, project


def memory(root: Path) -> Path:
    path = root / "notes"
    path.mkdir()
    for name, headers in [
        ("Plan", "ID | Status | Depends on | Feature | User capability"),
        ("Decisions", "ID | Decision | Applies in"),
        ("Invariants", "ID | Invariant | Enforced by"),
    ]:
        (path / f"{name}.md").write_text(f"# {name}\n\n| {headers} |\n")
        (path / name).mkdir()
    (path / "State.md").write_text(
        "# State\n\n"
        + "\n\n".join(
            f"## {section}\n\nNone."
            for section in [
                "Focus",
                "Workspace",
                "Progress",
                "Verification",
                "Blockers",
                "Next action",
            ]
        )
    )
    return path


def test_memory_and_read_only_resume(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    path = memory(tmp_path)
    assert invoke(worker, tmp_path, "memory-check").returncode == 0
    state = path / "State.md"
    state.write_text(
        state.read_text().replace("## Workspace\n\nNone.", "## Workspace\n\nBranch: `missing`")
    )
    before = {file: file.read_bytes() for file in tmp_path.rglob("*") if file.is_file()}
    output = invoke(worker, tmp_path, "resume")
    assert output.returncode == 0
    assert json.loads(output.stdout)["snapshot"] == "stale"
    assert {file: file.read_bytes() for file in tmp_path.rglob("*") if file.is_file()} == before


def test_rust_decision_markers_are_comments(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    path = memory(tmp_path)
    (path / "Decisions.md").write_text(
        (path / "Decisions.md").read_text()
        + "| [D001](Decisions/001.md) | Choice | [code](../src/lib.rs) |\n"
    )
    (path / "Decisions/001.md").write_text(
        "# D001\n\n"
        + "\n\n".join(
            f"## {heading}\n\nText."
            for heading in ["Context", "Chosen", "Rejected", "Rationale", "Consequences"]
        )
    )
    source = tmp_path / "src/lib.rs"
    source.write_text('const TEXT: &str = "// DECISION: D001";')
    assert invoke(worker, tmp_path, "memory-check").returncode == 2
    source.write_text("// DECISION: D001\nfn example() {}")
    assert invoke(worker, tmp_path, "memory-check").returncode == 0
    source.unlink()
    result = invoke(worker, tmp_path, "memory-check")
    assert result.returncode == 2
    assert "missing or escaping memory link" in result.stderr


def test_plan_cycle(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    path = memory(tmp_path)
    for identity, dependency in [("001", "002"), ("002", "001")]:
        with (path / "Plan.md").open("a") as stream:
            stream.write(
                f"| [P{identity}](Plan/{identity}.md) | pending | P{dependency} | Result | Capability |\n"
            )
        (path / f"Plan/{identity}.md").write_text(
            "# Feature\n\n"
            + "\n\n".join(
                f"## {heading}\n\nText." for heading in ["Feature", "User capability", "Acceptance"]
            )
        )
    result = invoke(worker, tmp_path, "memory-check")
    assert result.returncode == 2
    assert "dependency cycle" in result.stderr


# INVARIANT: I003
def test_committed_decisions_checked_against_staged_memory(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    path = memory(tmp_path)
    config = tmp_path / "worker.toml"
    with config.open("a") as stream:
        stream.write(
            '\n[[checks]]\nid = "memory"\nkind = "memory"\nskill = "guides/repair/SKILL.md"\n'
        )
    index = path / "Decisions.md"
    index.write_text(
        index.read_text() + "| [D001](Decisions/001.md) | Choice | [code](../src/lib.rs) |\n"
    )
    detail = path / "Decisions/001.md"
    detail.write_text(
        "# D001\n\n"
        + "\n\n".join(
            f"## {heading}\n\nText."
            for heading in ["Context", "Chosen", "Rejected", "Rationale", "Consequences"]
        )
    )
    (tmp_path / "src/lib.rs").write_text("// DECISION: D001\nfn example() {}")
    for args in [
        ("init", "-q"),
        ("config", "user.name", "Test"),
        ("config", "user.email", "test@example.invalid"),
        ("add", "."),
        ("commit", "-qm", "baseline"),
    ]:
        subprocess.run(["git", *args], cwd=tmp_path, check=True, capture_output=True)
    assert invoke(worker, tmp_path, "memory-check").returncode == 0
    old_index = index.read_text()
    index.write_text(old_index.replace("| Choice |", "| Changed |"))
    assert (
        "committed decision identity cannot change"
        in invoke(worker, tmp_path, "memory-check").stderr
    )
    # Refactoring changes current ownership without mutating historical rationale.
    (tmp_path / "src/lib.rs").rename(tmp_path / "src/moved.rs")
    index.write_text(old_index.replace("src/lib.rs", "src/moved.rs"))
    assert invoke(worker, tmp_path, "memory-check").returncode == 0
    (tmp_path / "src/moved.rs").rename(tmp_path / "src/lib.rs")
    index.write_text(old_index)
    original = detail.read_text()
    detail.write_text(original.replace("Text.", "Changed."))
    subprocess.run(["git", "add", str(detail)], cwd=tmp_path, check=True)
    detail.write_text(original)
    assert invoke(worker, tmp_path, "memory-check").returncode == 0
    result = invoke(worker, tmp_path, "check", "--staged")
    assert result.returncode == 2
    assert "committed decision detail cannot change" in result.stderr
    index.write_text(
        index.read_text().replace(
            "| [D001](Decisions/001.md) | Choice | [code](../src/lib.rs) |\n", ""
        )
    )
    result = invoke(worker, tmp_path, "memory-check")
    assert result.returncode == 2
    assert "committed decision cannot be removed" in result.stderr


@pytest.mark.parametrize(
    ("suffix", "code", "marker"),
    [("rs", "fn example() {}", "//"), ("py", "value = 1", "#"), ("sh", "echo example", "#")],
)
# INVARIANT: I001
def test_decision_source_markers(
    worker: Path, tmp_path: Path, suffix: str, code: str, marker: str
) -> None:
    project(tmp_path)
    notes = memory(tmp_path)
    index = notes / "Decisions.md"
    index.write_text(
        index.read_text()
        + f"| [D001](Decisions/001.md) | Choice | [source](../src/example.{suffix}) |\n"
    )
    (notes / "Decisions/001.md").write_text(
        "# D001\n\n"
        + "\n\n".join(
            f"## {heading}\n\nChoice."
            for heading in ["Context", "Chosen", "Rejected", "Rationale", "Consequences"]
        )
    )
    source = tmp_path / f"src/example.{suffix}"
    source.write_text(code)
    assert "marker absent" in invoke(worker, tmp_path, "memory-check").stderr
    source.write_text(f"{marker} DECISION: D001\n{code}")
    result = invoke(worker, tmp_path, "memory-check")
    assert result.returncode == 0, result.stderr


# INVARIANT: I008
def test_repository_layout_and_detail_contract(worker: Path, tmp_path: Path) -> None:
    root = Path(__file__).parents[4]
    assert (root / "Project/README.md").is_file()
    assert (root / "Ledger/Plan.md").is_file()
    project(tmp_path)
    notes = memory(tmp_path)
    (notes / "Plan/001.md").write_text("# Unindexed")
    assert "unindexed detail" in invoke(worker, tmp_path, "memory-check").stderr
    (notes / "Plan/001.md").unlink()
    assert invoke(worker, tmp_path, "memory-check").returncode == 0
