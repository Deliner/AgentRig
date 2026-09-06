import json
import subprocess
from pathlib import Path

import pytest
from test_lint import CONFIG, prepare


def inventory_project(root: Path, vcs: str) -> None:
    source = root / "src"
    source.mkdir(parents=True)
    for name in ["tracked.py", "deleted.py"]:
        (source / name).write_text("line\n" * 6)
    subprocess.run([vcs, "init"], cwd=root, capture_output=True, check=True)
    subprocess.run([vcs, "add", "src"], cwd=root, capture_output=True, check=True)
    git = vcs == "git"
    args = (
        ["-c", "user.name=Test", "-c", "user.email=test@example.invalid", "commit", "-qm", "base"]
        if git
        else ["commit", "-m", "base", "-u", "Test"]
    )
    subprocess.run([vcs, *args], cwd=root, capture_output=True, check=True)
    ignore = root / (".gitignore" if git else ".hgignore")
    ignore.write_text(
        "ignored.py\ntracked.py\n" if git else "syntax: glob\n**/ignored.py\n**/tracked.py\n"
    )
    for name in ["added.py", "unknown.py", "ignored.py"]:
        (source / name).write_text("line\n" * 6)
    subprocess.run([vcs, "add", "src/added.py"], cwd=root, capture_output=True, check=True)
    (source / "deleted.py").unlink()
    (source / "link.py").symlink_to("tracked.py")
    (root / (".git" if git else ".hg") / "control.py").write_text("line\n" * 6)


@pytest.mark.parametrize("vcs", ["git", "hg"])
def test_both_linters_use_native_tracked_and_untracked_inventory(
    worker: Path, tmp_path: Path, vcs: str
) -> None:
    policy = tmp_path / "policy"
    policy.mkdir()
    prepare(policy, 'skill_root: "."\n' + CONFIG.replace('include: ["src/**"]', 'include: ["**"]'))
    root = tmp_path / "consumer"
    inventory_project(root, vcs)
    before = {path.name: path.read_bytes() for path in (root / "src").iterdir()}
    findings = []
    for binary in [worker, worker.with_name("agentrig-lint")]:
        embedded = binary == worker
        prefix = ["lint"] if embedded else []
        result = subprocess.run(
            [
                str(binary),
                *prefix,
                "--root",
                str(root),
                "--config",
                str(policy / "lint.yaml"),
                "--json",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        assert result.returncode == 1, result.stdout + result.stderr
        items = json.loads(result.stdout)
        assert {item["path"] for item in items} == {
            "src/tracked.py",
            "src/added.py",
            "src/unknown.py",
        }
        for item in items:
            item.pop("rerun")
        findings.append(items)
    assert findings[0] == findings[1]
    assert {path.name: path.read_bytes() for path in (root / "src").iterdir()} == before
