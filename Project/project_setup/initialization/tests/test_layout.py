import json
import subprocess
from pathlib import Path
from typing import Any

import pytest
import yaml

from Project.shared.testing.consumer import (
    MODULES,
    file_contents,
    invoke,
    read_config,
)

AGENTRIG = """[package]
name = "agentrig"

[lib]
path = "lib.rs"

[[bin]]
name = "agentrig"
path = "command_line/main.rs"

[[bin]]
name = "agentrig-lint"
path = "structural_lint/standalone.rs"
"""

PARENT_PATHS = """[package]
name = "x"

[lib]
path = "../shared/lib.rs"

[[bin]]
name = "outside"
path = "../../../outside.rs"
"""


# DECISION: D040
def test_default_layout_keeps_memory_in_ledger_and_sources_in_project(
    worker: Path, tmp_path: Path
) -> None:
    assert invoke(worker, tmp_path, "init").returncode == 0
    paths = read_config(tmp_path)["paths"]
    assert (paths["memory"], paths["sources"]) == ("Ledger", ["Project/**"])
    assert (tmp_path / "Ledger/State.md").is_file()


def test_rust_init_derives_a_binary_crate_from_the_root_manifest(
    worker: Path, tmp_path: Path
) -> None:
    package(tmp_path, "tool", "main.rs")
    result = invoke(worker, tmp_path, "init", "--language", "rust", "--source", "src")
    assert result.returncode == 0, result.stderr
    assert rust_layout(tmp_path) == {
        "roots": ["src/main.rs"],
        "crates": {},
        "test": ["cargo", "test", "--manifest-path", "Cargo.toml"],
    }
    findings = json.loads(invoke(worker, tmp_path, "lint", "--json").stdout)
    messages = "\n".join(item["message"] for item in findings)
    assert "crate root is missing" not in messages
    assert "no successfully analyzed Rust crate root" not in messages


@pytest.mark.parametrize(("name", "crate"), [("a", "a"), ("core-a", "core_a")])
def test_rust_init_derives_workspace_member_roots(
    worker: Path, tmp_path: Path, name: str, crate: str
) -> None:
    (tmp_path / "Cargo.toml").write_text('[workspace]\nmembers = ["crates/*"]\n')
    package(tmp_path / "crates/a", name, "lib.rs")
    package(tmp_path / "crates/b", "b", "main.rs")
    result = invoke(worker, tmp_path, "init", "--language", "rust", "--source", "crates")
    assert result.returncode == 0, result.stderr
    assert rust_layout(tmp_path) == {
        "roots": ["crates/a/src/lib.rs", "crates/b/src/main.rs"],
        "crates": {crate: "crates/a/src/lib.rs"},
        "test": ["cargo", "test", "--manifest-path", "Cargo.toml"],
    }


def test_rust_init_keeps_the_agentrig_layout(worker: Path, tmp_path: Path) -> None:
    roots = [
        "Project/lib.rs",
        "Project/command_line/main.rs",
        "Project/structural_lint/standalone.rs",
    ]
    for root in roots:
        (tmp_path / root).parent.mkdir(parents=True, exist_ok=True)
        (tmp_path / root).write_text("")
    (tmp_path / "Project/Cargo.toml").write_text(AGENTRIG)
    result = invoke(worker, tmp_path, "init", "--language", "rust")
    assert result.returncode == 0, result.stderr
    assert rust_layout(tmp_path) == {
        "roots": roots,
        "crates": {"agentrig": "Project/lib.rs"},
        "test": ["cargo", "test", "--manifest-path", "Project/Cargo.toml"],
    }


def test_rust_init_expands_recursive_member_globs(worker: Path, tmp_path: Path) -> None:
    (tmp_path / "Cargo.toml").write_text('[workspace]\nmembers = ["crates/**"]\n')
    package(tmp_path / "crates/a/b", "b", "lib.rs")
    package(tmp_path / "crates/c", "c", "main.rs")
    result = invoke(worker, tmp_path, "init", "--language", "rust", "--source", "crates")
    assert result.returncode == 0, result.stderr
    assert rust_layout(tmp_path) == {
        "roots": ["crates/a/b/src/lib.rs", "crates/c/src/main.rs"],
        "crates": {"b": "crates/a/b/src/lib.rs"},
        "test": ["cargo", "test", "--manifest-path", "Cargo.toml"],
    }


def test_rust_init_accepts_parent_paths_inside_the_root(worker: Path, tmp_path: Path) -> None:
    root = tmp_path / "repo"
    (root / "crates/x").mkdir(parents=True)
    (root / "crates/shared").mkdir()
    (root / "crates/shared/lib.rs").write_text("")
    (tmp_path / "outside.rs").write_text("")
    (root / "Cargo.toml").write_text('[workspace]\nmembers = ["crates/x"]\n')
    (root / "crates/x/Cargo.toml").write_text(PARENT_PATHS)
    result = invoke(worker, root, "init", "--language", "rust", "--source", "crates")
    assert result.returncode == 0, result.stderr
    assert rust_layout(root) == {
        "roots": ["crates/shared/lib.rs"],
        "crates": {"x": "crates/shared/lib.rs"},
        "test": ["cargo", "test", "--manifest-path", "Cargo.toml"],
    }


def test_rust_init_skips_excluded_workspace_members(worker: Path, tmp_path: Path) -> None:
    (tmp_path / "Cargo.toml").write_text(
        '[workspace]\nmembers = ["crates/*"]\nexclude = ["crates/skip"]\n'
    )
    package(tmp_path / "crates/a", "a", "lib.rs")
    package(tmp_path / "crates/skip", "skip", "lib.rs")
    result = invoke(worker, tmp_path, "init", "--language", "rust", "--source", "crates")
    assert result.returncode == 0, result.stderr
    assert rust_layout(tmp_path) == {
        "roots": ["crates/a/src/lib.rs"],
        "crates": {"a": "crates/a/src/lib.rs"},
        "test": ["cargo", "test", "--manifest-path", "Cargo.toml"],
    }


def package(directory: Path, name: str, root: str) -> None:
    (directory / "src").mkdir(parents=True)
    (directory / "Cargo.toml").write_text(f'[package]\nname = "{name}"\nversion = "0.1.0"\n')
    (directory / "src" / root).write_text("")


def rust_layout(root: Path) -> dict[str, Any]:
    lint = yaml.safe_load((root / ".agentrig/lint.yaml").read_text())
    settings: dict[str, Any] = {}
    for rule in lint["rules"]:
        architecture = rule["id"] == "architecture"
        if architecture:
            settings = rule["architecture"]
    return {
        "roots": settings["rust_roots"],
        "crates": settings.get("rust_crates", {}),
        "test": read_config(root)["commands"]["test"]["argv"],
    }


@pytest.mark.parametrize("case", ["invalid-glob", "file-parent", "generated-parent"])
def test_init_rejects_invalid_layout_before_writing(
    worker: Path, tmp_path: Path, case: str
) -> None:
    args = ["--source", "src/["]
    existing_file_parent = case == "file-parent"
    generated_file_parent = case == "generated-parent"
    if existing_file_parent:
        (tmp_path / ".codex").write_text("user data")
        args = []
    elif generated_file_parent:
        args = ["--skills", ".agentrig/bin/agentrig"]
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "init", *args)
    assert result.returncode == 2
    assert file_contents(tmp_path) == before


# DECISION: D039
@pytest.mark.parametrize("language", ["python", "rust"])
def test_custom_service_executes_installed_adapters(
    worker: Path, tmp_path: Path, language: str
) -> None:
    service = "rig space's $cash"
    subprocess.run(["git", "init", "-q"], cwd=tmp_path, check=True)
    result = invoke(worker, tmp_path, "init", "--language", language, "--service", service)
    assert result.returncode == 0, result.stderr
    modules = {f"{name}.yaml" for name in ("project", *MODULES)}
    assert {path.name for path in (tmp_path / ".agentrig").iterdir()} == modules
    assert not (tmp_path / ".worker").exists()
    installed = tmp_path / service / "bin/agentrig"
    assert invoke(installed, tmp_path, "doctor").returncode == 0
    checked = subprocess.run(
        ["just", "config-check"], cwd=tmp_path, capture_output=True, text=True, check=False
    )
    assert checked.returncode == 0, checked.stderr
    hook = json.loads((tmp_path / ".codex/hooks.json").read_text())
    command = hook["hooks"]["SessionStart"][0]["hooks"][0]["command"]
    started = subprocess.run(
        ["sh", "-c", command],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
        input=json.dumps({"hook_event_name": "SessionStart", "session_id": "custom-service"}),
    )
    assert started.returncode == 0, started.stderr
    assert "complexity" in started.stdout
    blocked = subprocess.run(
        [str(tmp_path / service / "hooks/pre-commit")],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
    )
    assert blocked.returncode != 0
    assert "feature" in blocked.stderr
    assert (tmp_path / service / "runtime/reminders").is_dir()


@pytest.mark.parametrize("service", ["../escape", "bad\npath", "{{injection}}"])
def test_invalid_service_leaves_consumer_untouched(
    worker: Path, tmp_path: Path, service: str
) -> None:
    before = file_contents(tmp_path)
    result = invoke(worker, tmp_path, "init", "--service", service)
    assert result.returncode == 2
    assert file_contents(tmp_path) == before
