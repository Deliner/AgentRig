from __future__ import annotations

import os
import subprocess
from pathlib import Path

ROOT = Path(__file__).parents[2]


def install_runner(root: Path) -> None:
    configured = os.environ.get("WORKER_BINARY")
    if configured:
        binary = Path(configured)
    else:
        subprocess.run([str(ROOT / "tooling/worker/run"), "lint-rules"], cwd=ROOT, check=True)
        binary = ROOT / ".cache/worker/release/discipline-worker"
    destination = root / "tooling/worker"
    destination.mkdir(parents=True, exist_ok=True)
    launcher = destination / "run"
    launcher.write_text(f'#!/bin/sh\nexec "{binary}" "$@"\n', encoding="utf-8")
    launcher.chmod(0o755)
