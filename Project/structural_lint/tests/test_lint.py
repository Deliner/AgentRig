from __future__ import annotations

import json
import shlex
import subprocess
from pathlib import Path

import pytest

# DECISION: D016
from Project.structural_lint.testing.consumer import CONFIG, SKILL, lint, prepare


@pytest.mark.parametrize(
    "case",
    [(3, None, 0), (4, "warning", 0), (5, "warning", 0), (6, "error", 1)],
)
# INVARIANT: I004
def test_native_lint_thresholds_and_skills(
    worker: Path, tmp_path: Path, case: tuple[int, str | None, int]
) -> None:
    lines, level, code = case
    prepare(tmp_path)
    for extension in ["rs", "py", "ts"]:
        (tmp_path / f"src/example.{extension}").write_text("line\n\n" * lines, encoding="utf-8")
    (tmp_path / "src/ignored.bin").write_bytes(b"\xff" * 100)
    actual_code, diagnostics = lint(worker, tmp_path)
    assert actual_code == code
    within_limit = level is None
    assert len(diagnostics) == (0 if within_limit else 3)
    for item in diagnostics:
        assert item["level"] == level
        assert item["skill"] == SKILL
        assert item["actual"] == lines
        assert item["rule"] == "source"


# DECISION: D049
def test_warnings_are_shown_only_on_request(worker: Path, tmp_path: Path) -> None:
    prepare(tmp_path)
    (tmp_path / "src/warned.rs").write_text("line\n" * 4, encoding="utf-8")
    (tmp_path / "src/failed.rs").write_text("line\n" * 6, encoding="utf-8")
    base = [str(worker), "lint", "--root", str(tmp_path), "--config", "lint.yaml"]

    def run(*flags: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run([*base, *flags], capture_output=True, text=True, check=False)

    default = run()
    assert default.returncode == 1
    assert "ERROR [source] src/failed.rs" in default.stdout
    assert "WARNING" not in default.stdout + default.stderr
    assert "WARNING [source] src/warned.rs" in run("--warnings").stdout
    (tmp_path / "src/failed.rs").unlink()
    quiet = run("--json")
    assert (quiet.returncode, quiet.stdout.strip()) == (0, "[]")


def test_rerun_reproduces_findings_with_their_flags(worker: Path, tmp_path: Path) -> None:
    prepare(tmp_path)
    (tmp_path / "src/warned.rs").write_text("line\n" * 4, encoding="utf-8")
    (tmp_path / "src/failed.rs").write_text("line\n" * 6, encoding="utf-8")
    code, findings = lint(worker, tmp_path)
    assert sorted(item["level"] for item in findings) == ["error", "warning"]
    for finding in findings:
        rerun = subprocess.run(
            shlex.split(finding["rerun"]), capture_output=True, text=True, check=False
        )
        assert (rerun.returncode, json.loads(rerun.stdout)) == (code, findings)


def test_missing_skill_is_named_in_the_finding(worker: Path, tmp_path: Path) -> None:
    prepare(tmp_path)
    (tmp_path / SKILL).unlink()
    code, items = lint(worker, tmp_path)
    assert (code, items[0]["rule"]) == (2, "configuration")
    assert str(tmp_path.resolve() / SKILL) in items[0]["message"]


def test_project_mode_reports_a_missing_skill_as_a_json_finding(
    worker: Path, tmp_path: Path
) -> None:
    def run(*args: str) -> subprocess.CompletedProcess[str]:
        argv = [str(worker), args[0], "--root", str(tmp_path), *args[1:]]
        return subprocess.run(argv, capture_output=True, text=True, check=False)

    assert run("init").returncode == 0
    skill = tmp_path.resolve() / ".agentrig/skills/remove-debug/SKILL.md"
    skill.unlink()
    result = run("lint", "--json")
    findings = json.loads(result.stdout)
    assert (result.returncode, [item["rule"] for item in findings]) == (2, ["configuration"])
    assert str(skill) in findings[0]["message"]
    rerun = subprocess.run(
        shlex.split(findings[0]["rerun"]), capture_output=True, text=True, check=False
    )
    assert (rerun.returncode, json.loads(rerun.stdout)) == (2, findings)
    text = run("lint")
    assert (text.returncode, text.stdout) == (2, "")
    assert "ERROR [lint]" in text.stderr
    assert str(skill) in text.stderr


def test_overrides_and_exclusions(worker: Path, tmp_path: Path) -> None:
    config = CONFIG.replace(
        'extensions: [".rs", ".py", ".ts"]',
        'extensions: [".rs", ".py", ".ts"]\n    exclude: ["src/excluded.rs"]',
    )
    config += """
    overrides:
      - include: ["src/**"]
        extensions: [".rs"]
        warning: 7
        error: 9
      - include: ["src/special.rs"]
        warning: 10
        error: 12
"""
    prepare(tmp_path, config)
    for name in ["ordinary.rs", "ordinary.py", "special.rs", "excluded.rs"]:
        (tmp_path / "src" / name).write_text("line\n" * 8, encoding="utf-8")
    code, items = lint(worker, tmp_path)
    assert code == 1
    assert {item["path"]: item["level"] for item in items} == {
        "src/ordinary.rs": "warning",
        "src/ordinary.py": "error",
    }


@pytest.mark.parametrize(
    ("old", "new"),
    [
        ('kind: "nonblank-lines"', 'kind: "unknown"'),
        ('target: "file"', 'target: "directory"'),
        ("warning: 3", "warning: 5"),
        ("warning: 3", "warning: -1"),
        ("warning: 3", "warn: 3"),
        ('include: ["src/**"]', 'include: ["["]'),
        ('include: ["src/**"]', "include: []"),
        ("version: 1", "version: 99"),
        (f'error_skill: "{SKILL}"', 'error_skill: "missing/SKILL.md"'),
        (f'warning_skill: "{SKILL}"', ""),
    ],
)
# INVARIANT: I014
def test_invalid_config_is_actionable(worker: Path, tmp_path: Path, old: str, new: str) -> None:
    prepare(tmp_path, CONFIG.replace(old, new))
    code, items = lint(worker, tmp_path)
    assert code == 2
    assert items[0]["rule"] == "configuration"
    assert items[0]["skill"].endswith("repair/SKILL.md")


def test_directory_counts_immediate_children(worker: Path, tmp_path: Path) -> None:
    config = CONFIG.replace("nonblank-lines", "directory-entries").replace(
        'target: "file"', 'target: "directory"'
    )
    config = config.replace('include: ["src/**"]', 'include: ["src"]').replace(
        'extensions: [".rs", ".py", ".ts"]', ""
    )
    prepare(tmp_path, config)
    (tmp_path / "src/group").mkdir()
    for index in range(20):
        (tmp_path / f"src/group/{index}.txt").touch()
    assert lint(worker, tmp_path) == (0, [])
    for index in range(5):
        (tmp_path / f"src/{index}.txt").touch()
    code, items = lint(worker, tmp_path)
    assert code == 1
    assert items[0]["actual"] == 6


def test_staged_files_and_config_are_isolated(worker: Path, tmp_path: Path) -> None:
    root = tmp_path / "repository"
    root.mkdir()
    prepare(root)
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    source = root / "src/example.rs"
    source.write_text("line\n" * 6, encoding="utf-8")
    subprocess.run(["git", "add", "."], cwd=root, check=True)
    source.write_text("line\n", encoding="utf-8")
    (root / "lint.yaml").write_text(CONFIG.replace("error: 5", "error: 50"), encoding="utf-8")
    assert lint(worker, root)[0] == 0
    snapshot = tmp_path / "staged"
    snapshot.mkdir()
    subprocess.run(
        ["git", "checkout-index", "--all", f"--prefix={snapshot}/"], cwd=root, check=True
    )
    code, items = lint(worker, snapshot)
    assert code == 1
    assert items[0]["actual"] == 6
    assert items[0]["limit"] == 5


def test_effective_override_thresholds_must_be_valid(worker: Path, tmp_path: Path) -> None:
    prepare(
        tmp_path, CONFIG + '\n    overrides:\n      - include: ["src/**"]\n        warning: 9\n'
    )
    (tmp_path / "src/example.rs").write_text("line", encoding="utf-8")
    assert lint(worker, tmp_path)[0] == 2


def test_directory_rule_rejects_extension_filter(worker: Path, tmp_path: Path) -> None:
    prepare(
        tmp_path,
        CONFIG.replace("nonblank-lines", "directory-entries").replace(
            'target: "file"', 'target: "directory"'
        ),
    )
    assert lint(worker, tmp_path)[0] == 2
