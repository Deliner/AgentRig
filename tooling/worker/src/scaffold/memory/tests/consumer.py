import subprocess
from pathlib import Path

from tooling.worker.src.scaffold.testing.consumer import (
    invoke,
    project,
    update_config,
    vcs_backend,
    vcs_executable,
)


def memory(root: Path) -> Path:
    path = root / "notes"
    path.mkdir()
    for name, headers in [
        ("Plan", "ID | Status | Depends on | Feature | User capability"),
        ("Decisions", "ID | Decision | Applies in"),
        ("Invariants", "ID | Invariant | Enforced by"),
    ]:
        (path / f"{name}.md").write_text(f"# {name}\n\n| {headers} |\n")
        (path / name).mkdir()
    (path / "State.md").write_text(
        "# State\n\n"
        + "\n\n".join(
            f"## {section}\n\nNone."
            for section in [
                "Focus",
                "Workspace",
                "Progress",
                "Verification",
                "Blockers",
                "Next action",
            ]
        )
    )
    return path


def committed_memory(worker: Path, tmp_path: Path, vcs: str = "git") -> Path:
    project(tmp_path)
    update_config(tmp_path / "agentrig.yaml", git={"backend": vcs_backend(vcs)})
    path = memory(tmp_path)
    config = tmp_path / "agentrig.yaml"
    with config.open("a") as stream:
        stream.write(
            '\nchecks:\n- id: "memory"\n  kind: "memory"\n  skill: "guides/repair/SKILL.md"\n'
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
    git_commands = [
        ("init", "-q"),
        ("config", "user.name", "Test"),
        ("config", "user.email", "test@example.invalid"),
        ("add", "."),
        ("commit", "-qm", "baseline"),
    ]
    using_git = vcs == "git"
    commands = (
        git_commands
        if using_git
        else [("init",), ("add", "."), ("commit", "-m", "baseline", "-u", "Test")]
    )
    for args in commands:
        subprocess.run([vcs_executable(vcs), *args], cwd=tmp_path, check=True, capture_output=True)
    assert invoke(worker, tmp_path, "memory-check").returncode == 0
    return path
