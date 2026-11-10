import json
import subprocess
from pathlib import Path

import pytest

from Project.structural_lint.architecture.tests.test_architecture import (
    CONFIG,
    consumer,
    contract,
)
from Project.structural_lint.testing.consumer import edges, lint, prepare

WIDE = "pub fn wide() -> i32 { crate::b::value() + crate::c::value() }\n"


def python_hub(root: Path, packages: int, filename: str) -> None:
    prepare(root, CONFIG.replace("EXTENSION", ".py").replace("ROOTS", "[]"))
    names = [f"p{index}" for index in range(packages)]
    for name in names:
        (root / "src" / name).mkdir()
        (root / "src" / name / "api.py").write_text("def value():\n    return 1\n")
    (root / "src/hub").mkdir()
    (root / "src/hub" / filename).write_text("".join(f"import {name}.api\n" for name in names))
    for name in [*names, "hub"]:
        contract(root / "src" / name)
    contract(root / "src")


def shared_hub(root: Path) -> None:
    prepare(root, CONFIG.replace("EXTENSION", ".py").replace("ROOTS", "[]"))
    (root / "src/hub").mkdir()
    (root / "src/hub/api.py").write_text("def value():\n    return 1\n")
    contract(root / "src/hub")
    with (root / "src/hub/architecture.yaml").open("a") as file:
        file.write("dependents: 1\n")
    for name in ["p0", "p1"]:
        (root / "src" / name).mkdir()
        (root / "src" / name / "use.py").write_text("\nimport hub.api\n")
        contract(root / "src" / name)
    contract(root / "src")


# DECISION: D079
# DECISION: D142
def test_dependents_budget_names_each_dependent_with_an_edge(worker: Path, tmp_path: Path) -> None:
    shared_hub(tmp_path)
    code, findings = lint(worker, tmp_path)
    named = ", ".join(
        f"src/{name} (src/{name}/use.py:2 uses src/hub/api.py)" for name in ["p0", "p1"]
    )
    expected = f"directory is depended on by 2 other directories (budget 1): {named}"
    located = [(item["path"], item["message"].split("; ")[0]) for item in findings]
    assert (code, located) == (1, [("src/hub/architecture.yaml", expected)]), findings


# DECISION: D143
def test_edges_command_lists_the_edges_the_budget_counts(worker: Path, tmp_path: Path) -> None:
    shared_hub(tmp_path)
    used = {f"src/{name}": [f"src/{name}/use.py:2 uses src/hub/api.py"] for name in ["p0", "p1"]}
    hub = json.loads(edges(worker, tmp_path, "src/hub", "--json").stdout)
    assert hub == {
        "path": "src/hub",
        "rules": [{"id": "architecture", "incoming": used, "outgoing": {}}],
    }
    _, findings = lint(worker, tmp_path)
    assert f"depended on by {len(used)} other directories" in findings[0]["message"]
    p0 = json.loads(edges(worker, tmp_path, "src/p0", "--json").stdout)
    assert p0["rules"][0]["outgoing"] == {"src/hub": used["src/p0"]}
    text = edges(worker, tmp_path, "src/hub").stdout
    assert (
        "  incoming: 2 directories\n    src/p0\n      src/p0/use.py:2 uses src/hub/api.py\n" in text
    )
    unselected = edges(worker, tmp_path, "docs")
    assert unselected.returncode != 0 and "selects docs as a directory" in unselected.stderr


TEST_ONLY_USERS = {
    "src/lib.rs": "pub mod hub; pub mod a; pub mod b; pub mod c; pub mod p;\n",
    "src/hub/mod.rs": "pub fn value() -> i32 { 7 }\n",
    "src/a/mod.rs": "#[cfg(test)]\nmod tests;\n",
    "src/a/tests.rs": "use crate::hub::value;\n",
    "src/b/mod.rs": "#[cfg(test)]\nmod tests {\n    use crate::hub::value;\n}\n",
    "src/c/mod.rs": '#[cfg(test)]\n#[path = "tests/check.rs"]\nmod check;\n',
    "src/c/tests/check.rs": "use crate::hub::value;\n",
    "src/p/mod.rs": "pub fn run() -> i32 { crate::hub::value() }\n",
}


# DECISION: D146
def test_dependents_budget_counts_only_production_uses(worker: Path, tmp_path: Path) -> None:
    prepare(tmp_path, CONFIG.replace("EXTENSION", ".rs").replace("ROOTS", "[src/lib.rs]"))
    for name, text in TEST_ONLY_USERS.items():
        (tmp_path / name).parent.mkdir(parents=True, exist_ok=True)
        (tmp_path / name).write_text(text)
    for name in ["src/hub", "src/a", "src/b", "src/c/tests", "src/c", "src/p", "src"]:
        contract(tmp_path / name)
    with (tmp_path / "src/hub/architecture.yaml").open("a") as file:
        file.write("dependents: 0\n")
    production = "src/p/mod.rs:1 uses src/hub/mod.rs"
    expected = f"directory is depended on by 1 other directories (budget 0): src/p ({production})"
    code, findings = lint(worker, tmp_path)
    located = [(item["path"], item["message"].split("; ")[0]) for item in findings]
    assert (code, located) == (1, [("src/hub/architecture.yaml", expected)]), findings
    hub = json.loads(edges(worker, tmp_path, "src/hub", "--json").stdout)
    assert hub["rules"][0]["incoming"] == {"src/p": [production]}


OVER_DEFAULT = ("src/hub/api.py", None, "file uses 11 other project directories (budget 10)")
OVER_SETTING = (
    "src/a/mod.rs",
    None,
    "file uses 2 other project directories (budget 1): src/b, src/c",
)


# DECISION: D121
@pytest.mark.parametrize(("packages", "expected"), [(10, []), (11, [OVER_DEFAULT])])
def test_fan_out_default_budget_flags_files_above_ten_directories(
    worker: Path, tmp_path: Path, packages: int, expected: list[tuple[str, None, str]]
) -> None:
    python_hub(tmp_path, packages, "api.py")
    code, findings = lint(worker, tmp_path)
    assert code == (1 if expected else 0), findings
    summary = [(item["path"], item.get("line"), item["message"].split(":")[0]) for item in findings]
    assert summary == expected, findings


# DECISION: D121
def test_fan_out_exempts_python_main_entry_point(worker: Path, tmp_path: Path) -> None:
    python_hub(tmp_path, 11, "__main__.py")
    assert lint(worker, tmp_path) == (0, [])


# DECISION: D121
@pytest.mark.parametrize(
    ("hub", "expected"), [("src/a/mod.rs", [OVER_SETTING]), ("src/lib.rs", [])]
)
def test_fan_out_setting_flags_wide_files_but_not_crate_roots(
    worker: Path, tmp_path: Path, hub: str, expected: list[tuple[str, None, str]]
) -> None:
    consumer(tmp_path, "rs")
    (tmp_path / "src/c").mkdir()
    (tmp_path / "src/c/mod.rs").write_text("pub fn value() -> i32 { 3 }\n")
    (tmp_path / "src/lib.rs").write_text("pub mod a; pub mod b; pub mod c;\n")
    source = tmp_path / hub
    source.write_text(source.read_text() + WIDE)
    contract(tmp_path / "src/c")
    contract(tmp_path / "src")
    config = tmp_path / "lint.yaml"
    config.write_text(
        config.read_text().replace(
            "rust_roots: [src/lib.rs]\n", "rust_roots: [src/lib.rs]\n      fan_out: 1\n"
        )
    )
    code, findings = lint(worker, tmp_path)
    assert code == (1 if expected else 0), findings
    located = [(item["path"], item.get("line"), item["message"].split(";")[0]) for item in findings]
    assert located == expected, findings
    catalog = subprocess.run(
        [str(worker), "lint-rules"], capture_output=True, text=True, check=True
    )
    rows = {item["kind"]: item for item in json.loads(catalog.stdout)}
    assert "fan_out" in rows["directory-architecture"]["architecture"]
