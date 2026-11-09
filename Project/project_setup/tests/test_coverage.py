# DECISION: D124
# DECISION: D138
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import pytest

from Project.shared.testing.consumer import invoke, project, update_config

CARGO = ["cargo", "test", "--lib", "--"]
PYTEST = ["uv", "run", "pytest"]
SILENT_RUST = "map it to '::' (every test)"
SILENT_PYTHON = "map it to '.' (every test)"
UNSELECTED = "the gate never selects it"
SOURCES = {
    "src/lib.rs": "pub mod core;\n",
    "src/core/mod.rs": "mod mesh_tests;\n",
    "src/core/mesh_tests.rs": "#[test]\nfn mesh() {}\n",
    "src/app.py": "VALUE = 1\n",
    "tests/test_b.py": "def test_b() -> None:\n    pass\n",
    "tests/unit/test_a.py": "def test_a() -> None:\n    pass\n",
}


@dataclass(frozen=True)
class Case:
    argv: list[str]
    groups: list[dict[str, Any]]
    findings: dict[str, str] = field(default_factory=dict)
    cwd: str = "."
    include: list[str] = field(default_factory=lambda: ["**"])


CASES = {
    "rust-covered": Case(CARGO, [{"include": ["src/**"], "targets": ["core::"]}]),
    "rust-named-test": Case(
        CARGO, [{"include": ["src/**"], "targets": ["core::mesh_tests::mesh"]}]
    ),
    "rust-unmatched-runs-everything": Case(
        CARGO, [{"include": ["src/core/**"], "targets": ["core::"]}]
    ),
    "rust-silent-source": Case(
        CARGO,
        [
            {"include": ["**"], "targets": []},
            {"include": ["src/core/**"], "targets": ["core::"]},
        ],
        {"src/lib.rs": SILENT_RUST},
    ),
    "rust-unselected-test": Case(
        CARGO,
        [{"include": ["src/**"], "targets": ["core::other::"]}],
        {"src/core/mesh_tests.rs": UNSELECTED},
    ),
    "rust-test-outside-check-include": Case(
        CARGO,
        [{"include": ["src/**"], "targets": ["core::other::"]}],
        include=["src/lib.rs"],
    ),
    "rust-unselected-test-inside-check-include": Case(
        CARGO,
        [{"include": ["src/**"], "targets": ["core::other::"]}],
        {"src/core/mesh_tests.rs": UNSELECTED},
        include=["src/core/**"],
    ),
    "python-covered": Case(PYTEST, [{"include": ["**"], "targets": ["tests"]}]),
    "python-nodes": Case(
        PYTEST,
        [{"include": ["**"], "targets": ["tests/unit/test_a.py::test_a", "tests/test_b.py"]}],
    ),
    "python-command-directory": Case(
        PYTEST, [{"include": ["**"], "targets": ["unit", "test_b.py"]}], cwd="tests"
    ),
    "python-silent-source": Case(
        PYTEST,
        [
            {"include": ["src/**"], "targets": []},
            {"include": ["tests/**"], "targets": ["tests"]},
        ],
        {"src/app.py": SILENT_PYTHON},
    ),
    "python-silent-source-outside-check-include": Case(
        PYTEST,
        [
            {"include": ["src/**"], "targets": []},
            {"include": ["tests/**"], "targets": ["tests"]},
        ],
        include=["tests/**"],
    ),
    "python-sibling-unselected": Case(
        PYTEST,
        [{"include": ["**"], "targets": ["tests/test_b.py"]}],
        {"tests/unit/test_a.py": UNSELECTED},
    ),
    "unknown-runner-unchecked": Case(["make", "check"], [{"include": ["**"], "targets": []}]),
}


@pytest.mark.parametrize("name", CASES)
def test_config_check_reports_paths_the_affected_map_leaves_untested(
    worker: Path, tmp_path: Path, name: str
) -> None:
    case = CASES[name]
    project(tmp_path)
    for path, text in SOURCES.items():
        (tmp_path / path).parent.mkdir(parents=True, exist_ok=True)
        (tmp_path / path).write_text(text)
    command = {"argv": case.argv, "accepts_args": True, "cwd": case.cwd}
    check = {
        "id": "tests",
        "kind": "command",
        "command": "test",
        "include": case.include,
        "skill": "guides/repair/SKILL.md",
        "affected": case.groups,
    }
    update_config(tmp_path, commands={"test": command}, checks=[check])
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == int(bool(case.findings)), result.stdout + result.stderr
    assert result.stderr.count("ERROR [tests]") == len(case.findings), result.stderr
    for path, fragment in case.findings.items():
        line = result.stderr.split(f"ERROR [tests] {path}: ", 1)[1].splitlines()[0]
        assert fragment in line
        assert "guides/repair/SKILL.md" in line
        assert "config-check" in line.split("RERUN: ", 1)[1]
    valid = not case.findings
    assert ("Scaffold configuration is valid" in result.stdout) == valid
