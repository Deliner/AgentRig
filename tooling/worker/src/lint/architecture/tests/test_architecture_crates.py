import subprocess
from pathlib import Path

import pytest
import yaml

from tooling.worker.src.lint.architecture.tests.test_architecture import consumer, contract
from tooling.worker.src.lint.architecture.tests.test_architecture_behavior import parity


@pytest.mark.parametrize("form", ["direct", "alias", "absolute"])
def test_local_crate_edges_match_compiled_consumers(
    worker: Path, tmp_path: Path, form: str
) -> None:
    source, _ = consumer(tmp_path, "rs")
    source.write_text(
        {
            "direct": "pub fn run() -> i32 { peer::value() }",
            "alias": "use peer as service; pub fn run() -> i32 { service::value() }",
            "absolute": "pub fn run() -> i32 { ::peer::value() }",
        }[form]
    )
    configuration = tmp_path / "lint.yaml"
    configuration.write_text(
        configuration.read_text().replace(
            "      rust_roots: [src/lib.rs]",
            "      rust_crates:\n        peer: src/b/mod.rs\n      rust_roots: [src/lib.rs, src/b/mod.rs]",
        )
    )
    compile_consumer(tmp_path)
    assert parity(worker, tmp_path) == (0, [])
    contract(tmp_path / "src/a")
    status, findings = parity(worker, tmp_path)
    assert status == 1 and any("forbidden" in item["message"] for item in findings)


def compile_consumer(tmp_path: Path) -> None:
    library = tmp_path / "libpeer.rlib"
    subprocess.run(
        [
            "rustc",
            "+1.98.1",
            "--edition=2021",
            "--crate-name",
            "peer",
            "--crate-type=lib",
            str(tmp_path / "src/b/mod.rs"),
            "-o",
            str(library),
        ],
        check=True,
    )
    entry = tmp_path / "src/lib.rs"
    entry.write_text(entry.read_text() + "fn main() { std::process::exit(a::run()); }")
    executable = tmp_path / "consumer"
    subprocess.run(
        [
            "rustc",
            "+1.98.1",
            "--edition=2021",
            str(entry),
            "--extern",
            f"peer={library}",
            "-o",
            str(executable),
        ],
        check=True,
    )
    assert subprocess.run([str(executable)], check=False).returncode == 7


@pytest.mark.parametrize(
    "failure, expected",
    [
        ("missing", "crate root is missing"),
        ("unlisted", "must appear in rust_roots"),
        ("escape", "project-relative"),
        ("cycle", "cyclic local Rust crate reference"),
        ("macro", "local crate macro expansion"),
        ("unknown", "unresolved Rust name Missing"),
    ],
)
def test_local_crates_cannot_hide_incomplete_analysis(
    worker: Path, tmp_path: Path, failure: str, expected: str
) -> None:
    source, target = consumer(tmp_path, "rs")
    source.write_text("use peer::Value; pub fn run() { Value::call(); }")
    target.write_text("pub struct Value;")
    config = tmp_path / "lint.yaml"
    document = yaml.safe_load(config.read_text())
    settings = document["rules"][0]["architecture"]
    settings["rust_roots"] = ["src/lib.rs", "src/b/mod.rs"]
    settings["rust_crates"] = {"peer": "src/b/mod.rs"}
    settings["external"] = {"rust": ["std", "peer", "serde"]}
    missing = failure == "missing"
    unlisted = failure == "unlisted"
    escape = failure == "escape"
    cycle = failure == "cycle"
    macro = failure == "macro"
    unknown = failure == "unknown"
    if missing:
        target.unlink()
    if unlisted:
        settings["rust_roots"] = ["src/lib.rs"]
    if escape:
        settings["rust_crates"]["peer"] = "../outside.rs"
    if cycle:
        target.write_text("pub use peer::Value;")
    if macro:
        settings["rust_crates"]["serde"] = "src/b/mod.rs"
        source.write_text("#[derive(serde::Deserialize)] pub struct Value;")
    if unknown:
        source.write_text("use peer::Missing;")
    config.write_text(yaml.safe_dump(document))
    status, findings = parity(worker, tmp_path)
    assert status != 0 and findings
    assert any(expected in item["message"] for item in findings), findings
