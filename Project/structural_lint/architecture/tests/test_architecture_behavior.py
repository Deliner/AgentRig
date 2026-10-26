import subprocess
from pathlib import Path
from typing import Any

from Project.structural_lint.architecture.tests.test_architecture import back_edge, snapshot
from Project.structural_lint.testing.consumer import lint


def run_consumer(root: Path, language: str) -> str:
    rust = language == "rs"
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
    command = [
        "python3",
        "-I",
        "-B",
        "-c",
        "import sys; sys.path.insert(0, sys.argv[1]); from a.api import run; print(run())",
        str(root / "src"),
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


def assert_back_edge_forms_cycle(worker: Path, root: Path, target: Path) -> None:
    back_edge(target, "rs")
    code, findings = parity(worker, root)
    assert code == 1, findings
    assert any("directory dependency cycle" in item["message"] for item in findings), findings
