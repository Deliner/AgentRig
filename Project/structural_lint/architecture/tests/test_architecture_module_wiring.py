import subprocess
from pathlib import Path

import pytest

from Project.structural_lint.architecture.tests.test_architecture import CONFIG, contract
from Project.structural_lint.testing.consumer import lint, prepare


def consumer(root: Path, case: str) -> None:
    config = CONFIG.replace("EXTENSION", ".rs").replace("ROOTS", "[src/lib.rs]")
    prepare(root, config)
    child = root / "src/child"
    child.mkdir()
    additions = {
        "call": "pub fn run() { child::consume(Shared); }\n",
        "reexport": "pub use child::consume;\n",
    }
    (root / "src/lib.rs").write_text(
        "pub mod child;\npub struct Shared;\n" + additions.get(case, "")
    )
    (child / "mod.rs").write_text("pub fn consume(_: crate::Shared) {}\n")
    contract(root / "src")
    contract(child)


@pytest.mark.parametrize("edition", ["integrated", "standalone"])
@pytest.mark.parametrize("case", ["declaration", "call", "reexport"])
def test_module_declaration_does_not_invent_use_cycles(
    worker: Path, tmp_path: Path, edition: str, case: str
) -> None:
    consumer(tmp_path, case)
    subprocess.run(
        [
            "rustc",
            "--edition=2024",
            "--crate-type=lib",
            str(tmp_path / "src/lib.rs"),
            "-o",
            str(tmp_path / "consumer.rlib"),
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    standalone = edition == "standalone"
    binary = worker.with_name("agentrig-lint") if standalone else worker
    code, findings = lint(binary, tmp_path)
    expected = {
        "declaration": [],
        "call": ["directory dependency cycle"],
        "reexport": ["directory dependency cycle"],
    }[case]
    assert code == bool(expected), findings
    assert [item["message"].split(":")[0].split(" src/")[0] for item in findings] == expected


@pytest.mark.parametrize("edition", ["integrated", "standalone"])
def test_explicit_module_path_outside_crate_directory_is_measured(
    worker: Path, tmp_path: Path, edition: str
) -> None:
    config = CONFIG.replace("EXTENSION", ".rs").replace("ROOTS", "[src/lib.rs]")
    config = config.replace("[src, 'src/**']", "[src, 'src/**', shared, 'shared/**']")
    prepare(tmp_path, config)
    shared = tmp_path / "shared"
    shared.mkdir()
    (shared / "api.rs").write_text("pub fn value() -> i32 { 42 }\n")
    (tmp_path / "src/lib.rs").write_text(
        '#[path = "../shared/api.rs"] mod shared;\npub fn run() -> i32 { shared::value() }\n'
    )
    contract(tmp_path / "src")
    contract(shared)
    standalone = edition == "standalone"
    binary = worker.with_name("agentrig-lint") if standalone else worker
    assert lint(binary, tmp_path) == (0, [])
    (shared / "api.rs").write_text("pub fn value() -> i32 { 42 }\npub use crate::run as back;\n")
    code, findings = lint(binary, tmp_path)
    assert code == 1
    assert any("directory dependency cycle" in item["message"] for item in findings), findings
