from __future__ import annotations

import subprocess
from pathlib import Path

import yaml

from Project.structural_lint.testing.consumer import CONFIG, lint, prepare

WIDTH = CONFIG.replace("nonblank-lines", "line-width")


def record(worker: Path, root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [str(worker), "lint-baseline", "--root", str(root), "--config", "lint.yaml"],
        capture_output=True,
        text=True,
        check=False,
    )


def write(root: Path, name: str, long_lines: int) -> None:
    (root / "src" / name).write_text("long line\n" * long_lines + "ok\n", encoding="utf-8")


def baseline(root: Path) -> object:
    return yaml.safe_load((root / "lint-baseline.yaml").read_text(encoding="utf-8"))


def commit(root: Path) -> None:
    identity = ["-c", "user.name=Test", "-c", "user.email=test@example.invalid"]
    for args in [["init", "-q"], ["add", "."], [*identity, "commit", "-qm", "state"]]:
        subprocess.run(["git", *args], cwd=root, capture_output=True, check=True)


# DECISION: D123
def test_baseline_accepts_recorded_errors_and_fails_when_a_path_gains_one(
    worker: Path, tmp_path: Path
) -> None:
    prepare(tmp_path, WIDTH)
    write(tmp_path, "a.rs", 2)
    assert lint(worker, tmp_path)[0] == 1
    assert record(worker, tmp_path).returncode == 0
    assert baseline(tmp_path) == {"source": {"src/a.rs": 2}}
    code, items = lint(worker, tmp_path)
    assert code == 0
    assert [item["level"] for item in items] == ["warning", "warning"]
    assert all(item["message"].endswith("accepted by the lint baseline") for item in items)
    write(tmp_path, "a.rs", 3)
    code, items = lint(worker, tmp_path)
    assert code == 1
    assert [item["level"] for item in items] == ["error"] * 3
    assert "accepts 2 of these 3 findings" in items[0]["message"]


# DECISION: D123
def test_baseline_suggests_lowering_and_recording_never_raises_a_count(
    worker: Path, tmp_path: Path
) -> None:
    prepare(tmp_path, WIDTH)
    write(tmp_path, "a.rs", 2)
    write(tmp_path, "b.rs", 1)
    assert record(worker, tmp_path).returncode == 0
    write(tmp_path, "a.rs", 1)
    command = [str(worker), "lint", "--root", str(tmp_path), "--config", "lint.yaml"]
    lowered = subprocess.run(command, capture_output=True, text=True, check=False)
    assert lowered.returncode == 0
    assert "fell below the baseline for 1 rule and path pairs" in lowered.stderr
    write(tmp_path, "b.rs", 2)
    write(tmp_path, "c.rs", 1)
    assert lint(worker, tmp_path)[0] == 1
    recorded = record(worker, tmp_path)
    assert recorded.returncode == 0
    assert "2 rule and path pairs exceed the baseline and were not raised" in recorded.stdout
    assert baseline(tmp_path) == {"source": {"src/a.rs": 1, "src/b.rs": 1}}
    assert lint(worker, tmp_path)[0] == 1


# DECISION: D139
def test_recording_restores_a_lowered_count_up_to_the_committed_baseline(
    worker: Path, tmp_path: Path
) -> None:
    prepare(tmp_path, WIDTH)
    write(tmp_path, "a.rs", 2)
    write(tmp_path, "b.rs", 1)
    assert record(worker, tmp_path).returncode == 0
    commit(tmp_path)
    write(tmp_path, "a.rs", 0)
    assert record(worker, tmp_path).returncode == 0
    assert baseline(tmp_path) == {"source": {"src/b.rs": 1}}
    write(tmp_path, "a.rs", 2)
    assert lint(worker, tmp_path)[0] == 1
    assert record(worker, tmp_path).returncode == 0
    assert baseline(tmp_path) == {"source": {"src/a.rs": 2, "src/b.rs": 1}}
    assert lint(worker, tmp_path)[0] == 0
    write(tmp_path, "a.rs", 3)
    write(tmp_path, "c.rs", 1)
    assert record(worker, tmp_path).returncode == 0
    assert baseline(tmp_path) == {"source": {"src/a.rs": 2, "src/b.rs": 1}}
    assert lint(worker, tmp_path)[0] == 1


# DECISION: D139
def test_recording_keeps_a_rerecorded_baseline_and_lowers_an_uncommitted_one(
    worker: Path, tmp_path: Path
) -> None:
    prepare(tmp_path, WIDTH)
    write(tmp_path, "a.rs", 2)
    commit(tmp_path)
    assert record(worker, tmp_path).returncode == 0
    write(tmp_path, "a.rs", 1)
    assert record(worker, tmp_path).returncode == 0
    write(tmp_path, "a.rs", 2)
    assert record(worker, tmp_path).returncode == 0
    assert baseline(tmp_path) == {"source": {"src/a.rs": 1}}
    commit(tmp_path)
    (tmp_path / "lint-baseline.yaml").unlink()
    write(tmp_path, "c.rs", 1)
    assert record(worker, tmp_path).returncode == 0
    assert record(worker, tmp_path).returncode == 0
    assert baseline(tmp_path) == {"source": {"src/a.rs": 2, "src/c.rs": 1}}
