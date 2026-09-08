import subprocess
from pathlib import Path

import pytest

from tooling.worker.src.lint.architecture.tests.test_architecture import consumer, contract
from tooling.worker.src.lint.architecture.tests.test_architecture_behavior import (
    parity,
    run_consumer,
)


@pytest.mark.parametrize("form", ["text", "bytes", "alias", "nested"])
def test_embedded_resource_edges_preserve_compiled_behavior(
    worker: Path, tmp_path: Path, form: str
) -> None:
    source, _ = consumer(tmp_path, "rs")
    (tmp_path / "src/b/data.txt").write_text("content")
    expressions = {
        "text": 'include_str!("../b/data.txt")',
        "bytes": 'include_bytes!(r#"../b/data.txt"#)',
        "alias": 'text!("../b/data.txt")',
        "nested": 'format!("{}", include_str!("../b/data.txt"))',
    }
    source.write_text(
        f"use std::include_str as text; pub fn run() -> i32 {{ {expressions[form]}.len() as i32 }}"
    )
    entry = tmp_path / "src/lib.rs"
    entry.write_text(entry.read_text() + "fn main() { std::process::exit(a::run()); }")
    contract(tmp_path / "src/b")
    assert run_consumer(tmp_path, "rs") == "7"
    assert parity(worker, tmp_path) == (0, [])
    contract(tmp_path / "src/a")
    code, findings = parity(worker, tmp_path)
    assert code == 1, findings
    assert any(
        "src/b/data.txt" in item["message"] and "forbidden" in item["message"] for item in findings
    )


@pytest.mark.parametrize("failure", ["missing", "dynamic", "escape", "ignored"])
def test_resource_resolution_cannot_hide_unverified_targets(
    worker: Path, tmp_path: Path, failure: str
) -> None:
    source, _ = consumer(tmp_path, "rs")
    arguments = {
        "missing": '"../b/missing.txt"',
        "dynamic": 'concat!("../b/", "data.txt")',
        "escape": '"../../../outside.txt"',
        "ignored": '"../b/data.txt"',
    }
    source.write_text(f"pub fn run() {{ let _ = include_str!({arguments[failure]}); }}")
    ignored = failure == "ignored"
    if ignored:
        (tmp_path / "src/b/data.txt").write_text("content")
        (tmp_path / ".gitignore").write_text("src/b/data.txt\n")
        subprocess.run(["git", "init", "-q", str(tmp_path)], check=True)
    code, findings = parity(worker, tmp_path)
    assert code == 1, findings
    assert any("incomplete" in item["message"] for item in findings)
