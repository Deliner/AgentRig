from __future__ import annotations

import subprocess
from pathlib import Path

from test_language import configure
from test_lint import lint


def test_staged_language_source_and_policy(worker: Path, tmp_path: Path) -> None:
    root = tmp_path / "repo"
    root.mkdir()
    configure(root, "named-if-condition")
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    source = root / "src/example.py"
    source.write_text("if ready and enabled: pass")
    subprocess.run(["git", "add", "."], cwd=root, check=True)
    source.write_text("if ready: pass")
    config = root / "lint.toml"
    config.write_text(config.read_text().replace('level = "error"', 'level = "warning"'))
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
    config = tmp_path / "lint.toml"
    original = config.read_text().replace('extensions = [".rs", ".py", ".pyi"]', "")
    numeric = original[original.index("[[rules]]") :].replace('id = "source"', 'id = "parameters"')
    numeric = numeric.replace('kind = "named-if-condition"', 'kind = "parameter-count"').replace(
        'level = "error"', "error = 1"
    )
    config.write_text(
        (original + numeric).replace(
            'include = ["src/**"]', 'include = ["src/**"]\nexclude = ["src/other.js"]'
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
