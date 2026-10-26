import subprocess
from pathlib import Path

from Project.src.scaffold.testing.consumer import invoke, memory, project, update_config


def committed_memory(worker: Path, tmp_path: Path) -> Path:
    project(tmp_path)
    path = memory(tmp_path)
    update_config(
        tmp_path, checks=[{"id": "memory", "kind": "memory", "skill": "guides/repair/SKILL.md"}]
    )
    index = path / "Decisions.md"
    index.write_text(
        index.read_text() + "| [D001](Decisions/001.md) | Choice | [code](../src/lib.rs) |\n"
    )
    detail = path / "Decisions/001.md"
    detail.write_text(
        "# D001\n\n"
        + "\n\n".join(
            f"## {heading}\n\nText."
            for heading in ["Context", "Chosen", "Rejected", "Rationale", "Consequences"]
        )
    )
    (tmp_path / "src/lib.rs").write_text("// DECISION: D001\nfn example() {}")
    for args in [
        ("init", "-q"),
        ("config", "user.name", "Test"),
        ("config", "user.email", "test@example.invalid"),
        ("add", "."),
        ("commit", "-qm", "baseline"),
    ]:
        subprocess.run(["git", *args], cwd=tmp_path, check=True, capture_output=True)
    assert invoke(worker, tmp_path, "memory-check").returncode == 0
    return path
