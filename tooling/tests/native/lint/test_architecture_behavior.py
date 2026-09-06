import subprocess
from pathlib import Path
from typing import Any

import pytest
from test_architecture import consumer, contract, snapshot
from test_lint import lint


def private_consumer(root: Path, language: str) -> tuple[Path, str]:
    source, api = consumer(root, language)
    typescript = language == "ts"
    rust = language == "rs"
    if typescript:
        source.write_text(source.read_text().replace("api.js", "api.ts"))
        api.write_text("export const value = (): number => 7;\n")
    original = source.read_text()
    private = api.with_name(f"private.{language}")
    api.rename(private)
    contract(root / "src/b", public=f"['{api.name}']")
    if rust:
        api.write_text("pub mod private; pub use private::value;\n")
        source.write_text(original.replace("b::value", "b::private::value"))
        entry = root / "src/lib.rs"
        entry.write_text(entry.read_text() + "fn main() { std::process::exit(a::run()); }\n")
    else:
        facades = {
            "py": "from b.private import value\n",
            "js": 'export { value } from "./private.js";\n',
            "ts": 'export { value } from "./private.ts";\n',
        }
        api.write_text(facades[language])
        source.write_text(original.replace("b.api", "b.private").replace("b/api.", "b/private."))
    return source, original


def run_consumer(root: Path, language: str) -> str:
    rust = language == "rs"
    python = language == "py"
    if rust:
        binary = root.parent / "consumer-bin"
        subprocess.run(
            ["rustc", "+1.98.1", "--edition=2021", str(root / "src/lib.rs"), "-o", str(binary)],
            capture_output=True,
            text=True,
            check=True,
        )
        result = subprocess.run([str(binary)], capture_output=True, text=True, check=False)
        return str(result.returncode)
    if python:
        command = [
            "python3",
            "-I",
            "-B",
            "-c",
            "import sys; sys.path.insert(0, sys.argv[1]); from a.api import run; print(run())",
            str(root / "src"),
        ]
    else:
        command = [
            "node",
            "--experimental-strip-types",
            "--input-type=module",
            "-e",
            f'import {{ run }} from "./src/a/api.{language}"; console.log(run());',
        ]
    return subprocess.run(
        command, cwd=root, capture_output=True, text=True, check=True
    ).stdout.strip()


def parity(worker: Path, root: Path) -> tuple[int, list[dict[str, Any]]]:
    results = []
    before = snapshot(root)
    for binary in [worker, worker.with_name("agentrig-lint")]:
        code, findings = lint(binary, root)
        for finding in findings:
            assert str(binary) in finding.pop("rerun")
        results.append((code, findings))
    assert results[0] == results[1]
    assert snapshot(root) == before
    return results[0]


@pytest.mark.parametrize("language", ["rs", "py", "js", "ts"])
def test_public_api_repair_preserves_real_consumer_behavior(
    worker: Path, tmp_path: Path, language: str
) -> None:
    root = tmp_path / "consumer"
    root.mkdir()
    source, repair = private_consumer(root, language)
    before = snapshot(root)
    assert run_consumer(root, language) == "7"
    assert snapshot(root) == before
    code, findings = parity(worker, root)
    assert code == 1 and findings
    assert all("private access" in item["message"] for item in findings), findings
    source.write_text(repair)
    assert parity(worker, root) == (0, [])
    assert run_consumer(root, language) == "7"
    for path, content in before.items():
        is_contract = path.endswith("architecture.yaml")
        if is_contract:
            assert (root / path).read_bytes() == content
