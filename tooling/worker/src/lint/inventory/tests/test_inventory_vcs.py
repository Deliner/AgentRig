import json
import subprocess
from pathlib import Path

from tooling.worker.src.lint.testing.consumer import CONFIG, prepare


def inventory_project(root: Path) -> None:
    source = root / "src"
    source.mkdir(parents=True)
    for name in ["tracked.py", "deleted.py"]:
        (source / name).write_text("line\n" * 6)
    identity = ["-c", "user.name=Test", "-c", "user.email=test@example.invalid"]
    for args in [["init"], ["add", "src"], [*identity, "commit", "-qm", "base"]]:
        subprocess.run(["git", *args], cwd=root, capture_output=True, check=True)
    (root / ".gitignore").write_text("ignored.py\ntracked.py\n")
    for name in ["added.py", "unknown.py", "ignored.py"]:
        (source / name).write_text("line\n" * 6)
    subprocess.run(["git", "add", "src/added.py"], cwd=root, capture_output=True, check=True)
    (source / "deleted.py").unlink()
    (source / "link.py").symlink_to("tracked.py")
    (root / ".git/control.py").write_text("line\n" * 6)


def test_both_linters_use_native_tracked_and_untracked_inventory(
    worker: Path, tmp_path: Path
) -> None:
    policy = tmp_path / "policy"
    policy.mkdir()
    prepare(policy, 'skill_root: "."\n' + CONFIG.replace('include: ["src/**"]', 'include: ["**"]'))
    root = tmp_path / "consumer"
    inventory_project(root)
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
