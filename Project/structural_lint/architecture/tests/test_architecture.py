import json
import subprocess
from itertools import product
from pathlib import Path

import pytest
import yaml

from Project.structural_lint.testing.consumer import explain, lint, prepare

SKILL = ".agents/skills/refactor-large-directory/SKILL.md"
CONFIG = f"""version: 1
config_skill: .agents/skills/repair/SKILL.md
rules:
  - id: architecture
    kind: directory-architecture
    target: directory
    include: [src, 'src/**']
    extensions: [EXTENSION]
    level: error
    architecture:
      python_root: src
      rust_roots: ROOTS
    warning_skill: {SKILL}
    error_skill: {SKILL}
"""


BACK_EDGES = {"rs": "pub use crate::a as back;\n", "py": "import a.api\n"}


def contract(directory: Path) -> None:
    files = {}
    directories = {}
    for path in directory.iterdir():
        regular = path.is_file() and path.name != "architecture.yaml"
        child = path.is_dir()
        if regular:
            files[path.name] = "Fixture source or resource"
        elif child:
            directories[path.name] = "Fixture child responsibility"
    (directory / "architecture.yaml").write_text(
        f"purpose: {directory.name} responsibility\n"
        + yaml.safe_dump({"files": files, "directories": directories})
    )


def consumer(root: Path, language: str) -> tuple[Path, Path]:
    rust = language == "rs"
    config = CONFIG.replace("EXTENSION", f".{language}").replace(
        "ROOTS", "[src/lib.rs]" if rust else "[]"
    )
    prepare(root, config)
    for name in ["a", "b"]:
        (root / "src" / name).mkdir()
    filename = "mod.rs" if rust else f"api.{language}"
    source, target = [root / "src" / name / filename for name in ["a", "b"]]
    if rust:
        (root / "src/lib.rs").write_text("pub mod a; pub mod b;\n")
        source.write_text("pub fn run() -> i32 { crate::b::value() }\n")
        target.write_text("pub fn value() -> i32 { 7 }\n")
    else:
        source.write_text("from b.api import value\ndef run():\n    return value()\n")
        target.write_text("def value():\n    return 7\n")
    contract(root / "src")
    contract(root / "src/a")
    contract(root / "src/b")
    return source, target


def back_edge(target: Path, language: str) -> None:
    target.write_text(target.read_text() + BACK_EDGES[language])


@pytest.mark.parametrize("client", product(["rs", "py"], [False, True]))
@pytest.mark.parametrize("violation", ["valid", "missing", "cycle"])
def test_architecture_cli_contracts(
    worker: Path, tmp_path: Path, client: tuple[str, bool], violation: str
) -> None:
    language, standalone = client
    _, target = consumer(tmp_path, language)
    missing = violation == "missing"
    cycle = violation == "cycle"
    if missing:
        (tmp_path / "src/a/architecture.yaml").unlink()
    elif cycle:
        back_edge(target, language)
    binary = worker.with_name("agentrig-lint") if standalone else worker
    code, findings = lint(binary, tmp_path)
    valid = violation == "valid"
    assert code == (0 if valid else 1), findings
    if valid:
        assert findings == []
    else:
        assert any(violation in item["message"] for item in findings), findings
        assert all(item["rule"] == "architecture" and item["skill"] == SKILL for item in findings)
        assert all(item["actual"] is None and item["limit"] is None for item in findings)


@pytest.mark.parametrize("edition", ["integrated", "standalone"])
def test_architecture_warning_level_and_nonmutation(
    worker: Path, tmp_path: Path, edition: str
) -> None:
    _, target = consumer(tmp_path, "py")
    back_edge(target, "py")
    policy = tmp_path / "lint.yaml"
    policy.write_text(policy.read_text().replace("level: error", "level: warning"))
    before = snapshot(tmp_path)
    standalone = edition == "standalone"
    binary = worker.with_name("agentrig-lint") if standalone else worker
    code, findings = lint(binary, tmp_path)
    assert code == 0 and len(findings) == 1, findings
    assert findings[0]["level"] == "warning"
    assert "directory dependency cycle: src/a -> src/b -> src/a" in findings[0]["message"]
    assert findings[0]["path"] == "src/b/api.py" and findings[0]["line"] == 3
    assert before == snapshot(tmp_path)


def snapshot(root: Path) -> dict[str, bytes]:
    result = {}
    for path in root.rglob("*"):
        regular = path.is_file()
        if regular:
            result[str(path.relative_to(root))] = path.read_bytes()
    return result


@pytest.mark.parametrize("source", ["from missing.api import value\n", "from b.api import\n"])
def test_architecture_incomplete_analysis_is_a_finding(
    worker: Path, tmp_path: Path, source: str
) -> None:
    path, _ = consumer(tmp_path, "py")
    path.write_text(source)
    code, findings = lint(worker, tmp_path)
    assert code == 1
    assert any("incomplete" in item["message"] and item["line"] == 1 for item in findings)


@pytest.mark.parametrize(
    "replacement",
    [
        ("extensions: [.py]", "extensions: [.go]"),
        ("architecture:", "architecture:\n      unknown: true"),
        ("rust_roots: []", "rust_roots: ['../lib.rs']"),
        ("extensions: [.py]", "extensions: []"),
        ("level: error", "level: error\n    error: 4"),
    ],
)
def test_architecture_config_check_rejects_invalid_settings(
    worker: Path, tmp_path: Path, replacement: tuple[str, str]
) -> None:
    consumer(tmp_path, "py")
    policy = tmp_path / "lint.yaml"
    policy.write_text(policy.read_text().replace(*replacement))
    result = subprocess.run(
        [str(worker), "lint-config-check", "--root", str(tmp_path), "--json"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 2, result.stdout + result.stderr
    assert json.loads(result.stdout)[0]["rule"] == "configuration"


def test_architecture_discovery_and_directory_explanation(worker: Path, tmp_path: Path) -> None:
    consumer(tmp_path, "py")
    catalog = subprocess.run(
        [str(worker), "lint-rules"], capture_output=True, text=True, check=True
    )
    rows = {item["kind"]: item for item in json.loads(catalog.stdout)}
    row = rows["directory-architecture"]
    assert set(row["languages"]) == {"rust", "python"}
    assert row["target"] == "directory" and row["parameters"]["type"] == "policy"
    assert set(row["architecture"]["resolution"]) == {"rust", "python"}
    selected = explain(worker, tmp_path, "src/a")["rules"][0]
    assert selected["selected"] and selected["extensions"] == [".py"]
    assert selected["architecture"]["python_root"] == "src"
    docs = tmp_path / "src/docs"
    docs.mkdir()
    (docs / "guide.md").write_text("No source in this directory.\n")
    selected_docs = explain(worker, tmp_path, "src/docs")["rules"][0]
    assert selected_docs["selected"]
    code, findings = lint(worker, tmp_path)
    assert code == 1, findings
    assert any(item["path"] == "src/docs/architecture.yaml" for item in findings)
    contract(docs)
    contract(tmp_path / "src")
    assert lint(worker, tmp_path) == (0, [])


def test_architecture_rejects_git_target_through_escaping_symlink(
    worker: Path, tmp_path: Path
) -> None:
    root = tmp_path / "consumer"
    root.mkdir()
    consumer(root, "py")
    subprocess.run(["git", "init", "-q", str(root)], check=True)
    subprocess.run(["git", "-C", str(root), "add", "src"], check=True)
    outside = tmp_path / "outside"
    (root / "src/b").rename(outside)
    (root / "src/b").symlink_to(outside, target_is_directory=True)
    policy = root / "lint.yaml"
    policy.write_text("exclude: ['src/b/**']\n" + policy.read_text())
    code, findings = lint(worker, root)
    assert code == 1
    assert any(
        "target escapes project root" in item["message"] and item["line"] == 1 for item in findings
    )


@pytest.mark.parametrize(("setting", "code"), [("true", 2), ("false", 0)])
def test_architecture_rust_root_validation(
    worker: Path, tmp_path: Path, setting: str, code: int
) -> None:
    consumer(tmp_path, "rs")
    policy = tmp_path / "lint.yaml"
    policy.write_text(
        policy.read_text().replace("rust_roots: [src/lib.rs]", "rust_roots: []")
        + f"    enabled: {setting}\n"
    )
    result = subprocess.run(
        [str(worker), "lint-config-check", "--root", str(tmp_path), "--json"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == code
    assert lint(worker, tmp_path)[0] == result.returncode
