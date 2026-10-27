from __future__ import annotations

import subprocess
from pathlib import Path

import pytest

from Project.structural_lint.testing.consumer import SKILL, configure, lint


@pytest.mark.parametrize(
    "example",
    [
        (
            "rs",
            "//",
            "// SAFETY: the caller checks bounds",
            "/// Documentation.",
            'const TEXT: &str = "// DECISION: D001 data";',
        ),
        ("py", "#", "# type: ignore", '"""Documentation."""', 'TEXT = "# DECISION: D001 data"'),
    ],
)
def test_comment_references_through_cli(
    worker: Path, tmp_path: Path, example: tuple[str, str, str, str, str]
) -> None:
    extension, lead, directive, documentation, data = example
    configure(tmp_path, "comment-reference")
    reference = f"{lead} DECISION: D001"
    lines = [reference, f"{lead} plain", f"{reference} because", directive, documentation, data]
    path = tmp_path / f"src/example.{extension}"
    path.write_text("\n".join(lines) + "\n")
    code, items = lint(worker, tmp_path)
    assert code == 1
    assert [(item["rule"], item["line"]) for item in items] == [("source", n) for n in [2, 3, 4, 5]]
    result = subprocess.run(
        [str(worker), "lint", "--root", str(tmp_path), "--config", "lint.yaml"],
        text=True,
        capture_output=True,
        check=False,
    )
    assert "ERROR [source] 4 locations" in result.stdout
    assert f"src/example.{extension}:2" in result.stdout
    assert SKILL in result.stdout and "RERUN:" in result.stdout
    path.write_text(f"{reference}\n{data}\n")
    assert lint(worker, tmp_path) == (0, [])


def test_staged_language_source_and_policy(worker: Path, tmp_path: Path) -> None:
    root = tmp_path / "repo"
    root.mkdir()
    configure(root, "named-if-condition")
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    source = root / "src/example.py"
    source.write_text("if ready and enabled: pass")
    subprocess.run(["git", "add", "."], cwd=root, check=True)
    source.write_text("if ready: pass")
    config = root / "lint.yaml"
    config.write_text(config.read_text().replace('level: "error"', 'level: "warning"'))
    assert lint(worker, root) == (0, [])
    snapshot = tmp_path / "staged"
    snapshot.mkdir()
    subprocess.run(
        ["git", "checkout-index", "--all", f"--prefix={snapshot}/"], cwd=root, check=True
    )
    code, items = lint(worker, snapshot)
    assert code == 1
    assert items[0]["line"] == 1
    assert items[0]["level"] == "error"


def test_shared_parse_error_and_extension_defaults(worker: Path, tmp_path: Path) -> None:
    configure(tmp_path, "named-if-condition")
    config = tmp_path / "lint.yaml"
    original = config.read_text().replace('extensions: [".rs", ".py", ".pyi"]', "")
    numeric = original[original.index("  - id:") :].replace('id: "source"', 'id: "parameters"')
    numeric = numeric.replace('kind: "named-if-condition"', 'kind: "parameter-count"').replace(
        'level: "error"', "error: 1"
    )
    config.write_text(
        (original + numeric).replace(
            'include: ["src/**"]', 'include: ["src/**"]\n    exclude: ["src/other.js"]'
        )
    )
    (tmp_path / "src/example.pyi").write_text("def example(one, two): ...")
    (tmp_path / "src/broken.rs").write_text("fn broken( {")
    (tmp_path / "src/other.js").write_text("not Rust or Python")
    code, items = lint(worker, tmp_path)
    assert code == 1
    assert len(items) == 2
    assert [item["rule"] for item in items] == ["syntax", "parameters"]
    assert items[1]["actual"] == 2
