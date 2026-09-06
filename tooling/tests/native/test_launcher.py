from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

# DECISION: D015
ROOT = Path(__file__).parents[3]


@pytest.mark.parametrize(
    "resource", ["src/main.rs", "review/config/review.yaml", "review/config/project.yml"]
)
def test_launcher_rebuilds_changed_content_even_with_old_mtime(
    tmp_path: Path, resource: str
) -> None:
    source = tmp_path / "repo/tooling/worker"
    payload = source / resource
    payload.parent.mkdir(parents=True)
    shutil.copy(ROOT / "tooling/worker/run", source / "run")
    (source / "Cargo.toml").write_text("manifest", encoding="utf-8")
    payload.write_text("ONE", encoding="utf-8")
    original_time = payload.stat().st_mtime_ns
    fake = tmp_path / "bin"
    fake.mkdir()
    compiler = fake / "cargo"
    fake_cargo(compiler, resource)
    target = tmp_path / "target"
    env = {**os.environ, "PATH": f"{fake}:{os.environ['PATH']}", "WORKER_TARGET_DIR": str(target)}
    for expected in ["ONE", "ONE", "TWO"]:
        source_changed = expected == "TWO"
        if source_changed:
            payload.write_text("TWO", encoding="utf-8")
            os.utime(payload, ns=(original_time, original_time))
        result = subprocess.run(
            [str(source / "run")], env=env, text=True, capture_output=True, check=True
        )
        assert result.stdout.strip() == expected
    assert (target / "builds").read_text(encoding="utf-8").splitlines() == ["build", "build"]


def fake_cargo(compiler: Path, resource: str) -> None:
    compiler.write_text(
        f"#!{sys.executable}\n"
        "import pathlib, sys\n"
        "args = sys.argv\n"
        "manifest = pathlib.Path(args[args.index('--manifest-path') + 1])\n"
        "target = pathlib.Path(args[args.index('--target-dir') + 1])\n"
        "binary = target / 'release/discipline-worker'\n"
        "binary.parent.mkdir(parents=True, exist_ok=True)\n"
        f"value = (manifest.parent / {resource!r}).read_text()\n"
        "binary.write_text('#!/bin/sh\\necho ' + value + '\\n')\n"
        "binary.chmod(0o755)\n"
        "with (target / 'builds').open('a') as log: log.write('build\\n')\n",
        encoding="utf-8",
    )
    compiler.chmod(0o755)
