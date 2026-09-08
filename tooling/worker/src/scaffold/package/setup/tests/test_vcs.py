import json
import shutil
import subprocess
from pathlib import Path

from tooling.worker.src.scaffold.package.setup.tests.consumer import declaration
from tooling.worker.src.scaffold.testing.consumer import (
    file_contents,
    invoke,
    update_config,
    vcs_backend,
)


def test_private_setup_preview_resolves_adapter_from_consumer_root(
    worker: Path, tmp_path: Path
) -> None:
    root = declaration(worker, tmp_path, "private rig")
    backend = vcs_backend("private")
    assert isinstance(backend, dict)
    shutil.copy2(backend["command"][-1], root / "adapter.py")
    backend["command"][-1] = "adapter.py"
    update_config(root / "agentrig.yaml", vcs={"backend": backend})
    before = file_contents(root)
    result = subprocess.run(
        [str(worker), "setup", "--root", str(root), "--preview"],
        cwd=tmp_path,
        capture_output=True,
        text=True,
    )
    assert result.returncode == 0, result.stderr
    preview = json.loads(result.stdout)
    assert preview["registrations"]["vcs"]["initialize"] is True
    assert file_contents(root) == before
    assert not (root / ".hg").exists()
    applied = invoke(worker, root, "setup")
    assert applied.returncode == 0, applied.stdout + applied.stderr
    installed = root / "private rig/bin/agentrig"
    assert invoke(installed, root, "doctor").returncode == 0
    repeated = invoke(installed, root, "setup", "--preview")
    assert repeated.returncode == 0, repeated.stderr
    assert json.loads(repeated.stdout)["files"] == []
    assert json.loads(repeated.stdout)["registrations"]["vcs"]["initialize"] is False
