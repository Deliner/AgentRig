from __future__ import annotations

import json
import os
import subprocess
from functools import lru_cache
from pathlib import Path
from typing import Any

# DECISION: D019
ROOT = Path(__file__).parents[2]


@lru_cache(maxsize=1)
def worker_binary() -> Path:
    configured = os.environ.get("WORKER_BINARY")
    if configured:
        return Path(configured)
    subprocess.run(
        [str(ROOT / "tooling/worker/run"), "lint-rules"], cwd=ROOT, check=True, capture_output=True
    )
    return ROOT / ".cache/worker/release/discipline-worker"


def dispatch(event: dict[str, Any], root: Path = ROOT) -> dict[str, Any] | None:
    result = subprocess.run(
        [str(worker_binary()), "hook", "--root", str(root)],
        input=json.dumps(event),
        text=True,
        capture_output=True,
        check=True,
    )
    return json.loads(result.stdout) if result.stdout else None


def install_runner(root: Path) -> None:
    binary = worker_binary()
    destination = root / "tooling/worker"
    destination.mkdir(parents=True, exist_ok=True)
    launcher = destination / "run"
    launcher.write_text(f'#!/bin/sh\nexec "{binary}" "$@"\n', encoding="utf-8")
    launcher.chmod(0o755)
