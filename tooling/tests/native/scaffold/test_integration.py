import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

import pytest
from support import git, invoke


@pytest.mark.parametrize(
    ("language", "base", "prefix", "source", "memory", "skills"),
    [
        ("python", "trunk", "topic/", "application", "notes", "guides"),
        ("rust", "release", "change/", "crates/engine", "knowledge", "policies"),
    ],
)
# INVARIANT: I002
def test_independent_project_delivery(
    worker: Path,
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    language: str,
    base: str,
    prefix: str,
    source: str,
    memory: str,
    skills: str,
) -> None:
    monkeypatch.setenv("PATH", str(Path(sys.executable).parent) + os.pathsep + os.environ["PATH"])
    git(tmp_path, "init", "-q", "-b", base)
    git(tmp_path, "config", "user.name", "Test")
    git(tmp_path, "config", "user.email", "test@example.invalid")
    (tmp_path / ".gitignore").write_text("**/target/\n**/__pycache__/\n**/.pytest_cache/\n")
    git(tmp_path, "add", ".")
    git(tmp_path, "commit", "-qm", "base")
    git(tmp_path, "switch", "-c", prefix + "bootstrap")
    result = invoke(
        worker,
        tmp_path,
        "init",
        "--language",
        language,
        "--source",
        source,
        "--memory",
        memory,
        "--skills",
        skills,
        "--base",
        base,
        "--prefix",
        prefix,
    )
    assert result.returncode == 0, result.stderr
    binary = tmp_path / ".worker/bin/discipline-worker"
    assert binary.read_bytes() == worker.read_bytes()
    source_dir = tmp_path / source
    example = Path(__file__).parents[3] / "worker/examples" / language / source
    shutil.copytree(
        example, source_dir, ignore=shutil.ignore_patterns("target", "__pycache__", ".pytest_cache")
    )
    if language == "python":
        oracle = f"{source}/test_sample.py::test_doubles"
        function, link = "test_doubles", f"../{source}/test_sample.py"
    else:
        oracle = "tests::doubles"
        function, link = "doubles", f"../{source}/src/lib.rs"
    config = tmp_path / "worker.toml"
    with config.open("a") as stream:
        stream.write(
            f'\n[oracles.I001]\ncheck = "tests"\nrunner = "{language if language == "python" else "cargo"}"\ntarget = "{oracle}"\n'.replace(
                'runner = "python"', 'runner = "pytest"'
            )
        )
    mem = tmp_path / memory
    (mem / "Invariants").mkdir()
    (mem / "Invariants.md").write_text(
        (mem / "Invariants.md").read_text()
        + f"| [I001](Invariants/001.md) | Doubles correctly | [{function}]({link}) |\n"
    )
    (mem / "Invariants/001.md").write_text(
        "# I001\n\n## Predicate\n\nDoubles correctly.\n\n## Oracle\n\nRun configured I001.\n"
    )
    for command in ["config-check", "doctor", "resume", "memory-check", "check"]:
        result = invoke(binary, tmp_path, command)
        assert result.returncode == 0, (command, result.stdout, result.stderr)
    lint_config = tmp_path / ".worker/lint.toml"
    lint_before = lint_config.read_text()
    if language == "rust":
        lint_before = lint_before.replace('level = "warning"', 'level = "error"')
        lint_config.write_text(lint_before)
    assert invoke(binary, tmp_path, "lint").returncode == 0
    unsupported = source_dir / "tool.sh"
    unsupported.write_text("if true; then echo example; fi\n")
    lint_config.write_text(
        lint_before
        + f'\n[[rules]]\nid = "unsupported"\nkind = "function-lines"\ntarget = "file"\ninclude = ["{source}/**"]\nextensions = [".sh"]\nwarning = 40\nwarning_skill = "{skills}/refactor-long-function/SKILL.md"\nerror_skill = "{skills}/refactor-long-function/SKILL.md"\n'
    )
    result = invoke(binary, tmp_path, "config-check")
    assert result.returncode == 2
    assert ".sh" in result.stderr and "handlers support" in result.stderr
    lint_config.write_text(
        lint_config.read_text().replace('extensions = [".sh"]', 'extensions = [".rs", ".py"]')
    )
    assert invoke(binary, tmp_path, "config-check").returncode == 0
    assert invoke(binary, tmp_path, "run", "test").returncode == 0
    for args in [["config-check"], ["run", "test"]]:
        result = subprocess.run(
            ["just", *args], cwd=tmp_path, text=True, capture_output=True, check=False
        )
        assert result.returncode == 0, result.stdout + result.stderr
    test_path = source_dir / ("test_sample.py" if language == "python" else "src/lib.rs")
    valid_source = test_path.read_text()
    marker = "# INVARIANT: I001" if language == "python" else "// INVARIANT: I001"
    unrelated = (
        "# INVARIANT: I001\ndef unrelated():\n    pass\n"
        if language == "python"
        else "// INVARIANT: I001\nfn unrelated() {}\n"
    )
    test_path.write_text(unrelated + valid_source.replace(marker, ""))
    result = invoke(binary, tmp_path, "memory-check")
    assert result.returncode == 2
    assert "marked oracle function" in result.stderr
    if language == "python":
        same_name = (
            "class Other:\n    # INVARIANT: I001\n    def test_doubles(self):\n        pass\n"
        )
    else:
        same_name = "mod other {\n    // INVARIANT: I001\n    fn doubles() {}\n}\n"
    test_path.write_text(same_name + valid_source.replace(marker, ""))
    result = invoke(binary, tmp_path, "memory-check")
    assert result.returncode == 2
    assert "marked oracle function" in result.stderr
    test_path.write_text(valid_source)
    if language == "python":
        test_path.write_text(valid_source + "\ndef test_doubles():\n    pass\n")
        result = invoke(binary, tmp_path, "memory-check")
        assert result.returncode == 2
        assert "ambiguous" in result.stderr
        test_path.write_text(valid_source)
    if language == "python":
        decorator = '@pytest.mark.parametrize("unused", [1, 2])'
        parameterized = "import pytest\n" + valid_source.replace(
            "def test_doubles()", "def test_doubles(unused)"
        )
        for replacement in [marker + "\n" + decorator, decorator + "\n" + marker]:
            test_path.write_text(parameterized.replace(marker, replacement))
            result = invoke(binary, tmp_path, "memory-check")
            assert result.returncode == 0, result.stderr
        test_path.write_text(valid_source)
    valid_config = config.read_text()
    missing_target = oracle.replace("::", "::Missing::", 1)
    config.write_text(valid_config.replace(f'target = "{oracle}"', f'target = "{missing_target}"'))
    result = invoke(binary, tmp_path, "memory-check")
    assert result.returncode == 2
    assert "marked oracle function" in result.stderr
    config.write_text(valid_config)
    undiscoverable = (
        "__test__ = False\n" + valid_source
        if language == "python"
        else valid_source.replace("#[test]", "")
    )
    test_path.write_text(undiscoverable)
    result = invoke(binary, tmp_path, "memory-check")
    assert result.returncode == 2
    assert "test discovery failed" in result.stderr or "was not discovered" in result.stderr
    test_path.write_text(valid_source)
    records = [
        json.loads(line)
        for line in (tmp_path / ".worker/runtime/commands.jsonl").read_text().splitlines()
    ]
    assert any("--collect-only" in row["argv"] or "--list" in row["argv"] for row in records)
    hook = {
        "hook_event_name": "PreToolUse",
        "tool_name": "Write",
        "tool_input": {"file_path": f"{memory}/State.md"},
        "cwd": str(tmp_path),
    }
    result = invoke(binary, tmp_path, "hook", input=json.dumps(hook))
    assert result.returncode == 0
    assert f"{skills}/edit-state/SKILL.md" in result.stdout
    hook.update(tool_name="exec_command", tool_input={"cmd": "just run unknown"})
    assert "deny" in invoke(binary, tmp_path, "hook", input=json.dumps(hook)).stdout
    hook["tool_input"] = {"cmd": "echo bypass"}
    result = invoke(binary, tmp_path, "hook", input=json.dumps(hook))
    assert result.returncode == 0
    assert "deny" in result.stdout
    hook["tool_input"] = {"cmd": "just run test"}
    assert "deny" not in invoke(binary, tmp_path, "hook", input=json.dumps(hook)).stdout
    git(tmp_path, "add", ".")
    result = subprocess.run(
        ["just", "check", "--staged"], cwd=tmp_path, text=True, capture_output=True, check=False
    )
    assert result.returncode == 0, result.stdout + result.stderr
    git(tmp_path, "commit", "-qm", "attach scaffold")
    result = subprocess.run(
        ["just", "feature-merge"], cwd=tmp_path, text=True, capture_output=True, check=False
    )
    assert result.returncode == 0, result.stdout + result.stderr
    assert git(tmp_path, "branch", "--show-current").stdout.strip() == base
    assert git(tmp_path, "branch", "-D", prefix + "bootstrap", success=False).returncode != 0
    assert invoke(binary, tmp_path, "feature-start", "next").returncode == 0
    assert git(tmp_path, "branch", "--show-current").stdout.strip() == prefix + "next"
    assert not (tmp_path / "tooling/worker/Cargo.toml").exists()
    assert invoke(binary, tmp_path, "init").returncode == 2


def test_init_collision_preserves_files(worker: Path, tmp_path: Path) -> None:
    path = tmp_path / "justfile"
    path.write_text("user contents")
    result = invoke(worker, tmp_path, "init")
    assert result.returncode == 2
    assert "collision" in result.stderr
    assert list(tmp_path.iterdir()) == [path]
    assert path.read_text() == "user contents"
