import json
import subprocess
from pathlib import Path
from typing import Any

import pytest

from tooling.tests.native.test_lint import CONFIG, lint, prepare


def explain(worker: Path, root: Path, path: str) -> dict[str, Any]:
    result = subprocess.run(
        [str(worker), "lint-explain", path, "--root", str(root), "--config", "lint.yaml", "--json"],
        capture_output=True,
        text=True,
        check=True,
    )
    value: dict[str, Any] = json.loads(result.stdout)
    return value


@pytest.mark.parametrize("standalone", [False, True])
def test_explain_effective_overrides(worker: Path, tmp_path: Path, standalone: bool) -> None:
    binary = worker.with_name("agentrig-lint") if standalone else worker
    policy = (
        CONFIG
        + '\n    overrides:\n      - include: ["src/**"]\n        warning: 7\n        error: 9\n'
    )
    policy += '      - include: ["src/special.py"]\n        error: 12\n'
    prepare(tmp_path, policy)
    (tmp_path / "src/special.py").write_text("line\n" * 13)
    row = explain(binary, tmp_path, "./src/special.py")["rules"][0]
    assert row["selected"] and row["reason"] == "selected"
    assert (row["warning"], row["error"], row["matched_overrides"]) == (7, 12, [0, 1])
    code, findings = lint(binary, tmp_path)
    assert code == 1 and findings[0]["limit"] == row["error"]
    assert (tmp_path / "src/special.py").read_text() == "line\n" * 13


@pytest.mark.parametrize(
    "case",
    [
        ("enabled: false\n", "disabled"),
        ('exclude: ["src/example.py"]\n', "rule exclude matches"),
        ('extensions: [".rs"]\n', "extension does not match"),
        ('include: ["other/**"]\n', "include does not match"),
    ],
)
def test_explain_selection_reasons(worker: Path, tmp_path: Path, case: tuple[str, str]) -> None:
    setting, reason = case
    key = setting.split(":")[0]
    retained = []
    for line in CONFIG.splitlines():
        keep_line = not line.strip().startswith(key + ":")
        if keep_line:
            retained.append(line)
    policy = "\n".join(retained)
    prepare(tmp_path, policy + "\n    " + setting)
    (tmp_path / "src/example.py").write_text("line\n" * 8)
    row = explain(worker, tmp_path, "src/example.py")["rules"][0]
    assert not row["selected"] and row["reason"] == reason
    assert lint(worker, tmp_path) == (0, [])


def test_explain_inventory_and_global_exclusions(worker: Path, tmp_path: Path) -> None:
    prepare(tmp_path, 'exclude: ["src/hidden.py"]\n' + CONFIG)
    (tmp_path / "src/hidden.py").write_text("line\n" * 8)
    assert (
        explain(worker, tmp_path, "src/hidden.py")["rules"][0]["reason"] == "global exclude matches"
    )
    assert "absent" in explain(worker, tmp_path, "src/missing.py")["rules"][0]["reason"]
    (tmp_path / "src/visible.py").touch()
    assert explain(worker, tmp_path, "src")["rules"][0]["reason"] == "target type does not match"


def test_explain_rejects_invalid_effective_policy(worker: Path, tmp_path: Path) -> None:
    prepare(
        tmp_path, CONFIG + '\n    overrides:\n      - include: ["src/**"]\n        warning: 9\n'
    )
    (tmp_path / "src/example.py").touch()
    result = subprocess.run(
        [str(worker), "lint-explain", "src/example.py", "--root", str(tmp_path)],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 2
    assert "invalid effective thresholds" in result.stderr
