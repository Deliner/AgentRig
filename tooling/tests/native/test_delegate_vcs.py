import json
import shutil
import subprocess
from pathlib import Path

import pytest

from tooling.tests.native.test_delegate_code import SCRIPT, code_consumer, code_request
from tooling.tests.native.test_delegate_run import call, consumer, terminal


def backend(external: bool) -> str | dict[str, list[str]]:
    if external:
        script = Path(__file__).resolve().parents[2] / "worker/examples/external_vcs.py"
        return {"command": ["python3", "-B", str(script)]}
    return "mercurial"


def mercurial(root: Path, external: bool) -> str:
    git = root / ".git"
    has_git_fixture = git.exists()
    if has_git_fixture:
        shutil.rmtree(git)
    for args in [["init"], ["add", "src"], ["commit", "-m", "input", "-u", "Test"]]:
        subprocess.run(["hg", *args], cwd=root, capture_output=True, check=True)
    config = root / "delegate.yaml"
    config.write_text("vcs: " + json.dumps(backend(external)) + "\n" + config.read_text())
    return subprocess.check_output(
        ["hg", "log", "-r", ".", "-T", "{node}"], cwd=root, text=True
    ).strip()


@pytest.mark.parametrize(
    "case", [("read", False), ("artifacts", False), ("read", True), ("artifacts", True)]
)
def test_selected_delegate_reads_committed_snapshot(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, case: tuple[str, bool]
) -> None:
    mode, external = case
    script = 'read value < /project/src/value.txt\ntest "$value" = before\n'
    script += "test ! -e /project/.hg\n! echo changed > /project/src/value.txt\n"
    script += "printf artifact > /work/asset.txt\nprintf '{\"ok\":true}' > /work/result.json\n"
    consumer(worker, tmp_path, monkeypatch, script)
    source = tmp_path / "src/value.txt"
    source.parent.mkdir()
    source.write_text("before\n")
    base = mercurial(tmp_path, external)
    config = tmp_path / "delegate.yaml"
    config.write_text(config.read_text().replace('mode: "artifacts"', f'mode: "{mode}"'))
    path = tmp_path / "request.json"
    request = json.loads(path.read_text())
    request["revision"] = base
    reading = mode == "read"
    if reading:
        request["contract"]["artifacts"] = {}
    path.write_text(json.dumps(request))
    source.write_text("uncommitted\n")
    identifier = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    result = terminal(worker, tmp_path, identifier)
    assert result["outcome"] == "PASS", result
    directory = tmp_path / ".agentrig/runtime/jobs" / identifier
    inputs = json.loads((directory / "inputs.json").read_text())
    assert inputs["vcs"] == backend(external)
    assert inputs["revision"] == base
    assert source.read_text() == "uncommitted\n"
    assert not (directory / "private").exists()


@pytest.mark.parametrize("case", [(True, False), (False, False), (True, True), (False, True)])
def test_selected_code_result_preserves_checkout_and_applies_to_base(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, case: tuple[bool, bool]
) -> None:
    passes, external = case
    code_consumer(worker, tmp_path, monkeypatch, "test ! -e /project/.hg\n" + SCRIPT)
    base = mercurial(tmp_path, external)
    code_request(tmp_path, base)
    path = tmp_path / "request.json"
    request = json.loads(path.read_text())
    should_fail = not passes
    if should_fail:
        request["contract"]["changes"]["checks"]["value"] = ["python3", "-c", "raise SystemExit(3)"]
    path.write_text(json.dumps(request))
    source = tmp_path / "src/value.txt"
    source.write_text("uncommitted\n")
    identifier = call(worker, tmp_path, "start", "delegate.yaml", "request.json")["run_id"]
    result = terminal(worker, tmp_path, identifier)
    assert result["outcome"] == ("PASS" if passes else "ERROR"), result
    assert result["code"]["verified"] is passes
    assert result["code"]["base"] == base
    assert result["code"]["vcs"] == backend(external)
    assert source.read_text() == "uncommitted\n"
    directory = tmp_path / ".agentrig/runtime/jobs" / identifier
    applied = tmp_path / "applied"
    subprocess.run(
        ["hg", "clone", "-r", base, str(tmp_path), str(applied)], capture_output=True, check=True
    )
    subprocess.run(
        ["hg", "import", "--no-commit", str(directory / "change.patch")],
        cwd=applied,
        capture_output=True,
        check=True,
    )
    assert (applied / "src/value.txt").read_text() == "after\n"
    assert not (directory / "private").exists()
