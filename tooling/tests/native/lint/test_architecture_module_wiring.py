import subprocess
from pathlib import Path

import pytest

from tooling.tests.native.lint.test_architecture import CONFIG, contract
from tooling.tests.native.test_lint import lint, prepare


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
    contract(root / "src", public="['**']")
    private = case == "private"
    forbidden = case == "forbidden"
    contract(
        child,
        allow="[]" if forbidden else "['src/lib.rs']",
        public="[]" if private else "['mod.rs']",
    )


@pytest.mark.parametrize("standalone", [False, True])
@pytest.mark.parametrize("case", ["declaration", "call", "reexport", "private", "forbidden"])
def test_module_declaration_preserves_boundaries_without_inventing_use_cycles(
    worker: Path, tmp_path: Path, standalone: bool, case: str
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
    binary = worker.with_name("agentrig-lint") if standalone else worker
    code, findings = lint(binary, tmp_path)
    expected = {
        "declaration": [],
        "call": ["directory dependency cycle"],
        "reexport": ["directory dependency cycle"],
        "private": ["private access"],
        "forbidden": ["forbidden dependency"],
    }[case]
    assert code == bool(expected), findings
    assert [item["message"].split(":")[0].split(" src/")[0] for item in findings] == expected


@pytest.mark.parametrize("standalone", [False, True])
def test_explicit_module_path_outside_crate_directory_keeps_boundaries(
    worker: Path, tmp_path: Path, standalone: bool
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
    contract(tmp_path / "src", allow="['shared/api.rs']")
    contract(shared, public="['api.rs']")
    binary = worker.with_name("agentrig-lint") if standalone else worker
    assert lint(binary, tmp_path) == (0, [])
    contract(shared, public="[]")
    code, findings = lint(binary, tmp_path)
    assert code == 1
    assert any("private access" in item["message"] for item in findings)
