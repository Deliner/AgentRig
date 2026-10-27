import json
import subprocess
from pathlib import Path

from Project.shared.testing.consumer import invoke, project, update_config


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
        (
            ["read", "--", "sh", "-c", "ls"],
            f"read does not run sh; {programs} Project commands: just list. Temporary code: just"
            " scratch -- TEXT",
        ),
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
    (tmp_path / ".agentrig/commands.yaml").write_text("read:\n  argv: []\n  accepts_args: true\n")
    result = invoke(worker, tmp_path, "config-check")
    assert result.returncode == 2
    assert "commands.read: empty argv requires programs" in result.stderr


def test_configured_views_compact_command_output(worker: Path, tmp_path: Path) -> None:
    project(tmp_path)
    script = (
        "import sys\nfor n in range(int(sys.argv[2])): print('noise', n)\n"
        "print('FAILED case')\nprint('done', sys.argv[1])\nsys.exit(int(sys.argv[1]))"
    )
    views = {"success": {"tail": 1}, "failure": {"matching": ["FAILED"], "tail": 1}}
    noisy = {"argv": ["python3", "-c", script], "accepts_args": True, "output": views}
    update_config(tmp_path, commands={"noisy": noisy})

    def summary(code: str, noise: str) -> tuple[int, list[str], Path]:
        result = invoke(worker, tmp_path, "run", "noisy", "--", code, noise)
        *lines, status = result.stdout.splitlines()
        prefix = f"noisy: exit {result.returncode}; log: "
        assert status.startswith(prefix), result.stdout
        return result.returncode, lines, Path(status.removeprefix(prefix))

    code, lines, log = summary("0", "5")
    assert (code, lines) == (0, ["done 0"])
    assert summary("0", "500")[:2] == (0, ["done 0"])
    assert "noise 4\nFAILED case\ndone 0\n" in (log / "stdout.log").read_text()
    assert summary("3", "500")[:2] == (3, ["FAILED case", "done 3"])


def test_scratch_runs_temporary_code_in_its_task_area(worker: Path, tmp_path: Path) -> None:
    assert invoke(worker, tmp_path, "init").returncode == 0
    probe = ["git", "symbolic-ref", "HEAD", "refs/heads/feature/probe"]
    subprocess.run(probe, cwd=tmp_path, check=True)
    text = 'echo found > "$AGENTRIG_SCRATCH/results/out" && pwd'
    result = invoke(worker, tmp_path, "scratch", "--", text)
    assert result.returncode == 0, result.stderr
    assert result.stdout.strip() == str(tmp_path)
    area = tmp_path / ".agentrig/runtime/scratch/feature/probe"
    assert (area / "results/out").read_text() == "found\n"
    assert invoke(worker, tmp_path, "scratch", "--", "exit 7").returncode == 7
    records = json.loads(invoke(worker, tmp_path, "jobs").stdout)
    record = {tuple(record["argv"]): record for record in records}[("sh", "-c", text)]
    assert (record["command"], record["branch"], record["lifetime"]) == (
        "scratch",
        "feature/probe",
        "task",
    )
