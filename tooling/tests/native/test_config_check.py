from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

import pytest
from test_language import configure
from test_lint import CONFIG, lint, prepare

# DECISION: D018


def check(worker: Path, root: Path) -> tuple[int, list[dict[str, Any]]]:
    result = subprocess.run(
        [str(worker), "lint-config-check", "--root", str(root), "--config", "lint.toml", "--json"],
        capture_output=True,
        text=True,
        check=False,
    )
    return result.returncode, json.loads(result.stdout)


@pytest.mark.parametrize(
    "selector", ['extensions = [".sh"]', 'include = ["src/*.sh"]', 'include = ["src/**"]']
)
# INVARIANT: I016
def test_config_rejects_unsupported_language(worker: Path, tmp_path: Path, selector: str) -> None:
    configure(tmp_path, "named-if-condition")
    path = tmp_path / "lint.toml"
    config = path.read_text().replace('extensions = [".rs", ".py", ".pyi"]', "")
    if selector.startswith("include"):
        config = config.replace('include = ["src/**"]', selector)
    else:
        config += selector + "\n"
    path.write_text(config)
    (tmp_path / "src/run.sh").write_text("if true; then echo ok; fi")
    for result in [check(worker, tmp_path), lint(worker, tmp_path)]:
        code, items = result
        assert code == 2
        assert "Rust" in items[0]["message"] and "Python" in items[0]["message"]
        assert ".sh" in items[0]["message"]
        assert items[0]["skill"].endswith("configure-linter/SKILL.md")


def test_config_check_does_not_parse_source(worker: Path, tmp_path: Path) -> None:
    configure(tmp_path, "named-if-condition")
    (tmp_path / "src/broken.rs").write_text("fn broken( {")
    assert check(worker, tmp_path) == (0, [])
    assert lint(worker, tmp_path)[0] == 1


@pytest.mark.parametrize(
    ("old", "new"),
    [
        ('kind = "nonblank-lines"', 'kind = "unknown"'),
        ('target = "file"', 'target = "directory"'),
        ("warning = 3", "warning = 5"),
        ('include = ["src/**"]', 'include = ["["]'),
        (
            'error_skill = ".agents/skills/refactor-large-file/SKILL.md"',
            'error_skill = "missing/SKILL.md"',
        ),
        ("version = 1", "version = ["),
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
    prepare(tmp_path, CONFIG + '\n[[rules.overrides]]\ninclude = ["src/**"]\nwarning = 9\n')
    (tmp_path / "src/file.py").write_text("")
    assert check(worker, tmp_path)[0] == 2


def test_disabled_rule_preserves_schema_validation(worker: Path, tmp_path: Path) -> None:
    configure(tmp_path, "named-if-condition")
    path = tmp_path / "lint.toml"
    config = path.read_text().replace('extensions = [".rs", ".py", ".pyi"]', "enabled = false")
    path.write_text(config)
    (tmp_path / "src/run.sh").write_text("echo ok")
    assert check(worker, tmp_path) == (0, [])
    assert lint(worker, tmp_path) == (0, [])
    path.write_text(config.replace('level = "error"', 'level = "fatal"'))
    assert check(worker, tmp_path)[0] == 2


def test_explicit_suffix_rejected_without_files(worker: Path, tmp_path: Path) -> None:
    configure(tmp_path, "named-if-condition")
    path = tmp_path / "lint.toml"
    path.write_text(path.read_text().replace('include = ["src/**"]', 'include = ["future/*.sh"]'))
    assert check(worker, tmp_path)[0] == 2


def test_catalog_reports_handler_extensions(worker: Path) -> None:
    result = subprocess.run([str(worker), "lint-rules"], capture_output=True, text=True, check=True)
    rules = {item["kind"]: item for item in json.loads(result.stdout)}
    for kind in ["named-if-condition", "function-lines", "parameter-count"]:
        assert rules[kind]["target"] == "file"
        assert rules[kind]["handlers"] == {"rust": [".rs"], "python": [".py", ".pyi"]}
        assert set(rules[kind]["extensions"]) == {".rs", ".py", ".pyi"}
