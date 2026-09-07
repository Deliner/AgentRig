from pathlib import Path

import pytest
import yaml
from test_architecture import CONFIG, contract
from test_architecture_behavior import parity
from test_lint import prepare


def documents(root: Path) -> Path:
    prepare(root, CONFIG.replace("EXTENSION", ".py").replace("ROOTS", "[]"))
    docs = root / "src/docs"
    docs.mkdir()
    (docs / "guide.md").write_text("Feature ownership guide\n")
    contract(docs)
    contract(root / "src")
    return docs


@pytest.mark.parametrize(
    "violation", ["valid", "contract", "file", "directory", "stale", "kind", "purpose", "empty"]
)
def test_source_free_inventory_contracts(worker: Path, tmp_path: Path, violation: str) -> None:
    docs = documents(tmp_path)
    path = docs / "architecture.yaml"
    parent = violation == "directory"
    if parent:
        path = docs.parent / "architecture.yaml"
    data = yaml.safe_load(path.read_text())
    mutations = {
        "file": lambda: data["files"].pop("guide.md"),
        "directory": lambda: data["directories"].pop("docs"),
        "stale": lambda: data["files"].update({"deleted.md": "Removed resource"}),
        "kind": lambda: data["directories"].update({"guide.md": "Wrong kind"}),
        "purpose": lambda: data.update(purpose="First line\nSecond line"),
        "empty": lambda: data["files"].update({"guide.md": "  "}),
    }
    mutation = mutations.get(violation)
    if mutation:
        mutation()
        path.write_text(yaml.safe_dump(data))
    missing = violation == "contract"
    if missing:
        path.unlink()
    code, findings = parity(worker, tmp_path)
    valid = violation == "valid"
    assert code == (0 if valid else 1), findings
    assert all(item["path"] == str(path.relative_to(tmp_path)) for item in findings)


def test_inventory_excludes_generated_subtree(worker: Path, tmp_path: Path) -> None:
    docs = documents(tmp_path)
    generated = docs / "generated"
    generated.mkdir()
    (generated / "output.txt").write_text("Generated output\n")
    policy = tmp_path / "lint.yaml"
    policy.write_text(
        "exclude: ['src/docs/generated', 'src/docs/generated/**']\n" + policy.read_text()
    )
    assert parity(worker, tmp_path) == (0, [])


def test_rule_exclusion_keeps_inventory_and_analysis_in_scope(worker: Path, tmp_path: Path) -> None:
    docs = documents(tmp_path)
    (docs / "generated.py").write_text("from unknown import missing\n")
    (docs / "external").mkdir()
    (docs / "external/guide.md").write_text("External documentation\n")
    policy = tmp_path / "lint.yaml"
    policy.write_text(
        policy.read_text().replace(
            "    extensions:",
            "    exclude: ['src/docs/generated.py', 'src/docs/external', 'src/docs/external/**']\n    extensions:",
        )
    )
    assert parity(worker, tmp_path) == (0, [])
