import json
import subprocess
from pathlib import Path

import pytest

from Project.structural_lint.architecture.tests.test_architecture import (
    consumer,
    contract,
)
from Project.structural_lint.testing.consumer import lint

POLICY = """      disallowed:
        - path: std::fs
          reason: filesystem access belongs to the owner
          replacement: crate::b
          allow: ['src/b/**']
"""
MESSAGE = (
    "`std::fs::read_to_string` is disallowed here: filesystem access belongs to the owner; "
    "use crate::b"
)


def disallowed_consumer(root: Path, policy: str) -> None:
    source, target = consumer(root, "rs")
    (root / "src/a/fs.rs").write_text("pub fn local() -> i32 { 1 }\n")
    source.write_text(
        "pub mod fs;\n"
        "pub fn run() -> i32 {\n"
        '    let _ = std::fs::read_to_string("x");\n'
        "    fs::local() + crate::b::value()\n"
        "}\n"
    )
    target.write_text('pub fn value() -> i32 { std::fs::metadata("x").is_ok() as i32 }\n')
    contract(root / "src/a")
    config = root / "lint.yaml"
    config.write_text(
        config.read_text().replace(
            "rust_roots: [src/lib.rs]\n", "rust_roots: [src/lib.rs]\n" + policy
        )
    )


# DECISION: D105
def test_disallowed_paths_flag_references_outside_allowed_files(
    worker: Path, tmp_path: Path
) -> None:
    disallowed_consumer(tmp_path, POLICY)
    code, findings = lint(worker, tmp_path)
    assert code == 1, findings
    located = [(item["path"], item["line"], item["message"]) for item in findings]
    assert located == [("src/a/mod.rs", 3, MESSAGE)], findings
    catalog = subprocess.run(
        [str(worker), "lint-rules"], capture_output=True, text=True, check=True
    )
    rows = {item["kind"]: item for item in json.loads(catalog.stdout)}
    assert "disallowed" in rows["directory-architecture"]["architecture"]


# DECISION: D105
@pytest.mark.parametrize(
    "broken",
    [
        ("path: std::fs", "path: 'std::*'"),
        ("path: std::fs", "path: os.path"),
        ("reason: filesystem access belongs to the owner", "reason: ''"),
        ("replacement: crate::b", "replacement: ''"),
        ("allow: ['src/b/**']", "allow: ['../b/**']"),
        ("allow: ['src/b/**']", "allow: ['src/[b/**']"),
        ("allow: ['src/b/**']", "allow: []\n          extra: true"),
    ],
)
def test_disallowed_config_check_rejects_invalid_entries(
    worker: Path, tmp_path: Path, broken: tuple[str, str]
) -> None:
    disallowed_consumer(tmp_path, POLICY.replace(*broken))
    result = subprocess.run(
        [str(worker), "lint-config-check", "--root", str(tmp_path), "--json"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 2, result.stdout + result.stderr
    assert json.loads(result.stdout)[0]["rule"] == "configuration"
