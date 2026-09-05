import io
import os
import shutil
import subprocess
import tarfile
from pathlib import Path

import pytest


@pytest.fixture(scope="session", params=["acaa0b3", "83a81d9"])
def predecessor(request: pytest.FixtureRequest, tmp_path_factory: pytest.TempPathFactory) -> Path:
    root = Path(os.environ.get("WORKER_SOURCE_ROOT", Path(__file__).parents[4]))
    revision = str(request.param)
    cache = root / ".cache/upgrade-tests" / revision
    binary = cache / "discipline-worker"
    cached = binary.is_file()
    if cached:
        return binary
    source = tmp_path_factory.mktemp("previous-release")
    archive = subprocess.run(
        ["git", "archive", revision, "tooling/worker"],
        cwd=root,
        capture_output=True,
        check=True,
    )
    with tarfile.open(fileobj=io.BytesIO(archive.stdout)) as package:
        package.extractall(source, filter="data")
    target = root / ".cache/upgrade-tests/build"
    subprocess.run(
        [
            "cargo",
            "+1.98.1",
            "build",
            "--release",
            "--locked",
            "--manifest-path",
            str(source / "tooling/worker/Cargo.toml"),
            "--target-dir",
            str(target),
        ],
        check=True,
    )
    cache.mkdir(parents=True, exist_ok=True)
    shutil.copy2(target / "release/discipline-worker", binary)
    return binary
