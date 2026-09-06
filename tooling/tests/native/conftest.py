# DECISION: D019
from __future__ import annotations

import os
import subprocess
from pathlib import Path

import pytest

ROOT = Path(__file__).parents[3]


@pytest.fixture(scope="session")
def worker() -> Path:
    configured = os.environ.get("WORKER_BINARY")
    if configured:
        return Path(configured)
    subprocess.run([str(ROOT / "tooling/worker/run"), "lint-rules"], cwd=ROOT, check=True)
    return ROOT / ".cache/worker/release/agentrig"
