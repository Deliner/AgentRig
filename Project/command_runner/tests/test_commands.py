import json
from pathlib import Path

from Project.shared.testing.consumer import invoke, project


def test_config_and_command_streams(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    assert invoke(worker, tmp_path, "config-check").returncode == 0
    result = invoke(worker, tmp_path, "run", "echo", "--", "a b", "$(no-shell)", input="input data")
    assert result.returncode == 0, result.stderr
    assert "['a b', '$(no-shell)']" in result.stdout
    assert "input data" in result.stdout
    assert result.stderr == "stderr\n"
    assert invoke(worker, tmp_path, "run", "fail").returncode == 23
    records = json.loads(invoke(worker, tmp_path, "jobs").stdout)
    assert sorted(record["exit_code"] for record in records) == [0, 23]
    assert not (tmp_path / ".runtime/commands.jsonl").exists()
    assert "fail 1 1" in invoke(worker, tmp_path, "report").stdout
    assert invoke(worker, tmp_path, "run", "unknown").returncode == 2
    assert invoke(worker, tmp_path, "run", "fail", "extra").returncode == 2


def test_read_isolation(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    target = tmp_path / "src/protected"
    target.write_text("original")
    result = invoke(
        worker, tmp_path, "run", "read", "--", "sh", "-c", "echo replaced > src/protected"
    )
    assert result.returncode != 0
    assert target.read_text() == "original"
    result = invoke(worker, tmp_path, "run", "read", "--", "cat", "src/protected")
    assert result.returncode == 0, result.stderr
    assert result.stdout == "original"


def test_base_programs_and_argument_forms(worker: Path, tmp_path: Path) -> None:
    assert invoke(worker, tmp_path, "init").returncode == 0
    assert invoke(worker, tmp_path, "run", "read", "--", "ls").returncode == 0
    assert invoke(worker, tmp_path, "run", "write", "--", "touch", "made").returncode == 0
    assert (tmp_path / "made").is_file()
    programs = "configured programs: cat, diff, find, git, grep, head, ls, rg, sed, stat, tail, wc."
    refusals = [
        (["read", "--", "sh", "-c", "ls"], f"read does not run sh; {programs} Project commands"),
        (["write", "--", "python3", "-c", "pass"], "write does not run python3; configured"),
        (["read", "--", "git", "-c", "core.pager=sh", "log"], "read runs git only with: status,"),
        (
            ["read", "--", "find", ".", "-exec", "cat", "{}", "+"],
            "read does not run find with -exec",
        ),
        (
            ["write", "--", "git", "rebase", "--exec", "true", "main"],
            "does not run git with --exec",
        ),
    ]
    for args, message in refusals:
        result = invoke(worker, tmp_path, "run", *args)
        assert result.returncode == 2, args
        assert message in result.stderr, result.stderr
