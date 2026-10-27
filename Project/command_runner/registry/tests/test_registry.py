from pathlib import Path

from Project.shared.testing.consumer import git, invoke, read_config, update_config

TOOL = "#!/bin/sh\necho report\n"
CONTRACT = "purpose: Produce project reports\nfiles:\n  report.sh: Summarize the project state\n"


def consumer(worker: Path, root: Path) -> None:
    assert invoke(worker, root, "init").returncode == 0
    (root / "tools").mkdir()
    (root / "tools/report.sh").write_text(TOOL)
    (root / "tools/architecture.yaml").write_text(CONTRACT)
    (root / "tools/README.md").write_text("# Reports\n")
    report = {
        "purpose": "Summarize the project state",
        "argv": ["sh", "tools/report.sh"],
        "doc": "tools/README.md",
    }
    update_config(root, commands={"report": report})
    settings = read_config(root)["tools"]
    settings["locations"].append("tools/*")
    update_config(root, tools=settings)


def commit(root: Path) -> str:
    git(root, "add", "-A")
    identity = ["-c", "user.name=Test", "-c", "user.email=test@example.invalid"]
    git(root, "-c", "core.hooksPath=/dev/null", *identity, "commit", "-qm", "state")
    return git(root, "rev-parse", "HEAD").stdout.strip()


def test_registry_lists_capabilities_then_removed_tools(worker: Path, tmp_path: Path) -> None:
    consumer(worker, tmp_path)
    (tmp_path / "tools/old.sh").write_text(TOOL)
    revision = commit(tmp_path)
    (tmp_path / "tools/old.sh").unlink()
    old = {"purpose": "Print the old report", "reason": "replaced by report"}
    probe = {"purpose": "Try a report layout", "reason": "experiment ended"}
    removed = {
        "old": {**old, "path": "tools/old.sh", "revision": revision},
        "probe": {**probe, "path": "tools/probe.sh"},
    }
    update_config(tmp_path, tools={"removed": removed})
    result = invoke(worker, tmp_path, "commands")
    assert result.returncode == 0, result.stderr
    lines = result.stdout.splitlines()
    start = lines.index("report: Summarize the project state")
    assert lines[start + 1 : start + 4] == [
        "  just run report",
        "  tools/report.sh: Summarize the project state; owner tools: Produce project reports",
        "  doc: tools/README.md",
    ]
    read = lines.index("read: Inspect files and Git history with configured programs")
    assert lines[read + 1 : read + 3] == [
        "  just run read -- PROGRAM ARGS; programs: cat, diff, find, git, grep, head, ls, rg, sed,"
        " stat, tail, wc",
        "report: Summarize the project state",
    ]
    test = lines.index("test: Run the project tests")
    assert lines[test + 2] == "  runs python3 -m pytest Project"
    assert any(line.startswith(".agentrig/hooks/pre-commit: ") for line in lines)
    assert lines[lines.index("Removed:") + 1 :] == [
        "old: Print the old report",
        f"  tools/old.sh: replaced by report; restore: git show {revision}:tools/old.sh",
        "probe: Try a report layout",
        "  tools/probe.sh: experiment ended; never committed; not recoverable",
    ]
    assert git(tmp_path, "show", f"{revision}:tools/old.sh").stdout == TOOL


def test_tools_check_reports_structure_before_merge(worker: Path, tmp_path: Path) -> None:
    consumer(worker, tmp_path)
    initial = commit(tmp_path)
    assert invoke(worker, tmp_path, "check", "--only", "tools").returncode == 0
    (tmp_path / "loose.sh").write_text(TOOL)
    (tmp_path / "tools/stray").write_text("data")
    (tmp_path / "tools/stray").chmod(0o755)
    (tmp_path / "tools/README.md").write_text("Run tools/report.sh\n")
    update_config(tmp_path, commands={"report": {"argv": ["sh", "tools/moved.sh"]}})
    settings = read_config(tmp_path)["tools"]
    settings["entries"].append("tools/gone/*")
    settings["removed"] = {
        "old": {"purpose": "Report", "reason": "moved", "path": "tools/report.sh"},
        "test": {"purpose": "", "reason": "", "path": "tools/never.sh", "revision": initial},
    }
    update_config(tmp_path, tools=settings)
    result = invoke(worker, tmp_path, "check", "--only", "tools")
    assert result.returncode == 1, result.stdout + result.stderr
    locations = ".agentrig/bin/*, .agentrig/hooks/*, tools/*"
    for path, message in [
        ("loose.sh", f"tool is outside tools.locations: {locations}"),
        ("tools/stray", "tool is not registered"),
        ("tools/report.sh", "tool is not registered"),
        (".agentrig/commands.yaml", "commands.report: purpose is required"),
        (".agentrig/commands.yaml", "commands.report: tools/moved.sh does not exist"),
        (".agentrig/tools.yaml", "tools.entries tools/gone/* matches no tool"),
        (".agentrig/tools.yaml", "tools.removed.old: tools/report.sh still exists"),
        (".agentrig/tools.yaml", "tools.removed.test: purpose and reason are required"),
        (".agentrig/tools.yaml", "tools.removed.test is also an active catalog command"),
        (".agentrig/tools.yaml", "tools.removed.test: revision does not contain tools/never.sh"),
        ("tools/README.md", "references removed tool tools/report.sh"),
    ]:
        assert f"  {path}: {message}" in result.stdout, result.stdout
    assert "develop-tools/SKILL.md" in result.stderr
    git(tmp_path, "add", "-A")
    staged = invoke(worker, tmp_path, "check", "--staged", "--only", "tools")
    assert "SKIP [tools]: checked before merge" in staged.stdout, staged.stdout + staged.stderr
