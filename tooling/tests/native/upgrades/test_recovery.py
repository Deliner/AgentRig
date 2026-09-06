# DECISION: D024
import json
import subprocess
import time
from pathlib import Path

from .test_upgrade_plan import invoke, prepare


def resolved_plan(worker: Path, predecessor: Path, root: Path) -> Path:
    subprocess.run(["git", "init", "-q"], cwd=root, check=True)
    path, plan = prepare(worker, predecessor, root)
    plan["files"]["guides/repair/SKILL.md"]["resolution"] = "keep"
    path.write_text(json.dumps(plan))
    return path


def test_reapply_after_rollback(worker: Path, predecessor: Path, tmp_path: Path) -> None:
    path = resolved_plan(worker, predecessor, tmp_path)
    assert invoke(worker, tmp_path, "upgrade", "apply", str(path)).returncode == 0
    assert invoke(worker, tmp_path, "upgrade", "rollback").returncode == 0
    assert invoke(worker, tmp_path, "upgrade", "apply", str(path)).returncode == 0
    assert list(path.parent.parent.glob("restored-*/operation/journal.json"))


def test_mixed_installation_is_recoverable(worker: Path, predecessor: Path, tmp_path: Path) -> None:
    path = resolved_plan(worker, predecessor, tmp_path)
    assert invoke(worker, tmp_path, "upgrade", "apply", str(path)).returncode == 0
    operation = path.parent.parent / "operation"
    frozen = json.loads((operation / "plan.json").read_text())
    before = frozen["files"]["worker.toml"]["before"]["sha256"]
    (tmp_path / "worker.toml").write_bytes((operation / "blobs" / before).read_bytes())
    (tmp_path / "agentrig.yaml").unlink()
    journal = json.loads((operation / "journal.json").read_text())
    journal["phase"] = "applying"
    journal["completed"] = [".worker/bin/discipline-worker"]
    (operation / "journal.json").write_text(json.dumps(journal))
    resumed = invoke(worker, tmp_path, "resume")
    assert resumed.returncode == 0, resumed.stderr
    assert json.loads(resumed.stdout)["upgrade"]["phase"] == "applying"
    assert invoke(worker, tmp_path, "config-check").returncode != 0
    assert "unfinished upgrade" in invoke(worker, tmp_path, "guard-commit").stderr
    event = json.dumps({"hook_event_name": "SessionStart"})
    hook = subprocess.run(
        [str(worker), "hook", "--root", str(tmp_path)],
        input=event,
        capture_output=True,
        text=True,
        check=False,
    )
    assert "Upgrade operation: applying" in hook.stdout
    recovered = subprocess.run(
        ["just", "upgrade", "apply", str(path)],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
    )
    assert recovered.returncode == 0, recovered.stdout + recovered.stderr


def wait_for_checks(path: Path, process: subprocess.Popen[str]) -> None:
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        assert process.poll() is None, "upgrade exited before interruption"
        present = path.is_file()
        if present:
            journal = json.loads(path.read_text())
            running_tests = (
                journal["phase"] == "validating"
                and path.parents[2].joinpath("test-started").is_file()
            )
            if running_tests:
                return
        time.sleep(0.01)
    raise AssertionError("upgrade did not reach project verification")


def test_interrupted_verification_resumes(worker: Path, predecessor: Path, tmp_path: Path) -> None:
    path = resolved_plan(worker, predecessor, tmp_path)
    source = tmp_path / "src"
    source.mkdir()
    test = source / "test_slow.py"
    test.write_text(
        "import time\nfrom pathlib import Path\n"
        "def test_slow():\n"
        "    Path('.worker/runtime/test-started').write_text('ready')\n"
        "    time.sleep(30)\n"
    )
    journal = path.parent.parent / "operation/journal.json"
    with (tmp_path / "upgrade.log").open("w") as log:
        process = subprocess.Popen(
            [str(worker), "upgrade", "apply", str(path), "--root", str(tmp_path)],
            stdout=log,
            stderr=log,
            text=True,
        )
        try:
            wait_for_checks(journal, process)
            process.terminate()
            assert process.wait(timeout=10) != 0
        finally:
            alive = process.poll() is None
            if alive:
                process.kill()
                process.wait()
    assert json.loads(journal.read_text())["phase"] == "validating"
    test.write_text("def test_slow():\n    assert True\n")
    result = invoke(worker, tmp_path, "upgrade", "apply", str(path))
    assert result.returncode == 0, result.stdout + result.stderr
