import json
import subprocess
from pathlib import Path

import pytest
from test_lint import CONFIG, prepare


@pytest.mark.parametrize("git_repository", [False, True])
def test_standalone_matches_worker_without_installing_files(
    worker: Path, tmp_path: Path, git_repository: bool
) -> None:
    policy = tmp_path / "policy"
    policy.mkdir()
    prepare(policy, 'skill_root = "."\n' + CONFIG)
    project = tmp_path / "consumer"
    (project / "src").mkdir(parents=True)
    source = project / "src/value.py"
    source.write_text("line\n" * 6)
    if git_repository:
        subprocess.run(["git", "init", "-q", str(project)], check=True)
    before = sorted(str(path.relative_to(project)) for path in project.rglob("*"))
    args = ["--root", str(project), "--config", str(policy / "lint.toml"), "--json"]
    results = []
    for binary in [worker, worker.with_name("discipline-lint")]:
        using_worker = binary == worker
        prefix = ["lint"] if using_worker else []
        result = subprocess.run(
            [str(binary), *prefix, *args], capture_output=True, text=True, check=False
        )
        assert result.returncode == 1, result.stdout + result.stderr
        findings = json.loads(result.stdout)
        assert findings[0]["skill"] == str(policy / ".agents/skills/refactor-large-file/SKILL.md")
        for finding in findings:
            assert str(binary) in finding.pop("rerun")
        results.append(findings)
    assert results[0] == results[1]
    assert source.read_text() == "line\n" * 6
    assert sorted(str(path.relative_to(project)) for path in project.rglob("*")) == before


def test_standalone_config_check_rejects_unsupported_language(worker: Path, tmp_path: Path) -> None:
    policy = tmp_path / "policy"
    policy.mkdir()
    config = CONFIG.replace('kind = "nonblank-lines"', 'kind = "function-lines"')
    prepare(policy, 'skill_root = "."\n' + config)
    project = tmp_path / "consumer"
    project.mkdir()
    result = subprocess.run(
        [
            str(worker.with_name("discipline-lint")),
            "lint-config-check",
            "--root",
            str(project),
            "--config",
            str(policy / "lint.toml"),
            "--json",
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    assert result.returncode == 2
    diagnostic = json.loads(result.stdout)[0]
    assert "incompatible" in diagnostic["message"]
    assert diagnostic["skill"] == str(policy / ".agents/skills/repair/SKILL.md")
    assert list(project.iterdir()) == []
