from __future__ import annotations

import hashlib
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
    shutil.copy(ROOT / "tooling/worker/build", source / "build")
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
            [str(source / "build")], env=env, text=True, capture_output=True, check=True
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
        "binary = target / 'release/agentrig'\n"
        "binary.parent.mkdir(parents=True, exist_ok=True)\n"
        f"value = (manifest.parent / {resource!r}).read_text()\n"
        "binary.write_text('#!/bin/sh\\necho ' + value + '\\n')\n"
        "binary.chmod(0o755)\n"
        "with (target / 'builds').open('a') as log: log.write('build\\n')\n",
        encoding="utf-8",
    )
    compiler.chmod(0o755)


def development_tree(root: Path) -> tuple[Path, dict[str, str]]:
    source = root / "tooling/worker"
    source.mkdir(parents=True)
    for name in ["run", "build"]:
        shutil.copy2(ROOT / "tooling/worker" / name, source / name)
    pin = root / "tooling/distribution/stable.txt"
    pin.parent.mkdir()
    pin.write_text("a" * 40 + "\n")
    (source / "Cargo.toml").write_text("deliberately invalid manifest\n")
    env = dict(os.environ)
    for key in ["WORKER_BINARY", "WORKER_SOURCE_ROOT", "WORKER_TARGET_DIR", "CARGO_TARGET_DIR"]:
        env.pop(key, None)
    return source, env


def test_stable_runtime_survives_candidate_build_failure(worker: Path, tmp_path: Path) -> None:
    source, env = development_tree(tmp_path)
    installed = tmp_path / ".cache/development" / ("a" * 40)
    subprocess.run([str(worker), "init", "--root", str(installed)], check=True)
    binary = installed / ".agentrig/bin/agentrig"
    before = hashlib.sha256(binary.read_bytes()).hexdigest()
    candidate = subprocess.run(
        [str(source / "build"), "--version"], env=env, capture_output=True, text=True
    )
    assert candidate.returncode != 0
    assert "Cargo.toml" in candidate.stderr
    for args in [["--version"], ["config-check", "--root", str(installed)]]:
        result = subprocess.run(
            [str(source / "run"), *args], env=env, capture_output=True, text=True
        )
        assert result.returncode == 0, result.stderr
    event = '{"hook_event_name":"SessionStart"}'
    hook = subprocess.run(
        [str(source / "run"), "hook", "--root", str(installed)],
        env=env,
        input=event,
        capture_output=True,
        text=True,
    )
    assert hook.returncode == 0, hook.stderr
    assert "complexity-discipline" in hook.stdout
    assert hashlib.sha256(binary.read_bytes()).hexdigest() == before


def test_missing_stable_runtime_never_builds_candidate(tmp_path: Path) -> None:
    source, env = development_tree(tmp_path)
    result = subprocess.run(
        [str(source / "run"), "--version"], env=env, capture_output=True, text=True
    )
    assert result.returncode == 2
    assert "just bootstrap" in result.stderr
    assert not (tmp_path / ".cache/worker").exists()
