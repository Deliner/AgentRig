from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

import pytest

from tooling.worker.src.lint.testing.consumer import CONFIG, configure, lint, prepare

# DECISION: D018


def check(worker: Path, root: Path) -> tuple[int, list[dict[str, Any]]]:
    result = subprocess.run(
        [str(worker), "lint-config-check", "--root", str(root), "--config", "lint.yaml", "--json"],
        capture_output=True,
        text=True,
        check=False,
    )
    return result.returncode, json.loads(result.stdout)


@pytest.mark.parametrize(
    "selector", ['extensions: [".sh"]', 'include: ["src/*.sh"]', 'include: ["src/**"]']
)
# INVARIANT: I016
def test_config_rejects_unsupported_language(worker: Path, tmp_path: Path, selector: str) -> None:
    configure(tmp_path, "named-if-condition")
    path = tmp_path / "lint.yaml"
    config = path.read_text().replace('extensions: [".rs", ".py", ".pyi"]', "")
    include_selector = selector.startswith("include")
    if include_selector:
        config = config.replace('include: ["src/**"]', selector)
    else:
        config += "    " + selector + "\n"
    path.write_text(config)
    (tmp_path / "src/run.sh").write_text("if true; then echo ok; fi")
    for result in [check(worker, tmp_path), lint(worker, tmp_path)]:
        code, items = result
        assert code == 2
        assert "Rust" in items[0]["message"] and "Python" in items[0]["message"]
        assert ".sh" in items[0]["message"]
        assert items[0]["skill"].endswith("repair/SKILL.md")


def test_config_check_does_not_parse_source(worker: Path, tmp_path: Path) -> None:
    configure(tmp_path, "named-if-condition")
    (tmp_path / "src/broken.rs").write_text("fn broken( {")
    assert check(worker, tmp_path) == (0, [])
    assert lint(worker, tmp_path)[0] == 1


@pytest.mark.parametrize(
    ("old", "new"),
    [
        ('kind: "nonblank-lines"', 'kind: "unknown"'),
        ('target: "file"', 'target: "directory"'),
        ("warning: 3", "warning: 5"),
        ('include: ["src/**"]', 'include: ["["]'),
        (
            'error_skill: ".agents/skills/refactor-large-file/SKILL.md"',
            'error_skill: "missing/SKILL.md"',
        ),
        ("version: 1", "version: ["),
    ],
)
def test_config_check_reports_schema_errors(
    worker: Path, tmp_path: Path, old: str, new: str
) -> None:
    prepare(tmp_path, CONFIG.replace(old, new))
    code, items = check(worker, tmp_path)
    assert code == 2
    assert items[0]["rule"] == "configuration"


def test_config_check_effective_override(worker: Path, tmp_path: Path) -> None:
    prepare(
        tmp_path, CONFIG + '\n    overrides:\n      - include: ["src/**"]\n        warning: 9\n'
    )
    (tmp_path / "src/file.py").write_text("")
    assert check(worker, tmp_path)[0] == 2


def test_disabled_rule_preserves_schema_validation(worker: Path, tmp_path: Path) -> None:
    configure(tmp_path, "named-if-condition")
    path = tmp_path / "lint.yaml"
    config = path.read_text().replace('extensions: [".rs", ".py", ".pyi"]', "enabled: false")
    path.write_text(config)
    (tmp_path / "src/run.sh").write_text("echo ok")
    assert check(worker, tmp_path) == (0, [])
    assert lint(worker, tmp_path) == (0, [])
    path.write_text(config.replace('level: "error"', 'level: "fatal"'))
    assert check(worker, tmp_path)[0] == 2


def test_explicit_suffix_rejected_without_files(worker: Path, tmp_path: Path) -> None:
    configure(tmp_path, "named-if-condition")
    path = tmp_path / "lint.yaml"
    path.write_text(path.read_text().replace('include: ["src/**"]', 'include: ["future/*.sh"]'))
    assert check(worker, tmp_path)[0] == 2


def test_catalog_reports_handler_extensions(worker: Path) -> None:
    result = subprocess.run([str(worker), "lint-rules"], capture_output=True, text=True, check=True)
    rules = {item["kind"]: item for item in json.loads(result.stdout)}
    for kind in ["named-if-condition", "function-lines", "parameter-count"]:
        assert rules[kind]["target"] == "file"
        assert rules[kind]["handlers"] == {"rust": [".rs"], "python": [".py", ".pyi"]}
        assert set(rules[kind]["extensions"]) == {".rs", ".py", ".pyi"}


@pytest.mark.parametrize(
    "kind",
    [
        "nonblank-lines",
        "directory-entries",
        "named-if-condition",
        "function-lines",
        "parameter-count",
    ],
)
def test_catalog_examples_validate(worker: Path, tmp_path: Path, kind: str) -> None:
    example = subprocess.run(
        [str(worker), "lint-rule", kind, "--example"], capture_output=True, text=True, check=True
    ).stdout
    prepare(tmp_path, example)
    assert check(worker, tmp_path) == (0, [])
    details = subprocess.run(
        [str(worker), "lint-rule", kind, "--json"], capture_output=True, text=True, check=True
    )
    item = json.loads(details.stdout)
    assert item["kind"] == kind
    assert (tmp_path / ".agents/skills" / item["skill"] / "SKILL.md").is_file()
    policy = kind == "named-if-condition"
    assert item["parameters"]["type"] == ("policy" if policy else "thresholds")
    human = subprocess.run(
        [str(worker), "lint-rule", kind], capture_output=True, text=True, check=True
    ).stdout
    assert item["metric"] in human
    assert "--example" in human


@pytest.mark.parametrize(
    "args",
    [
        ["lint-rule"],
        ["lint-rule", "unknown"],
        ["lint-rule", "function-lines", "--typo"],
        ["lint-rules", "extra"],
    ],
)
def test_discovery_rejects_invalid_arguments(worker: Path, args: list[str]) -> None:
    result = subprocess.run([str(worker), *args], capture_output=True, text=True, check=False)
    assert result.returncode == 2
    assert result.stderr
