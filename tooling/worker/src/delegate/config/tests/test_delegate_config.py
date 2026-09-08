import subprocess
from pathlib import Path

import pytest

CONFIG = """schema_version: 1
profiles:
  reader:
    frontend: "codex"
    model: "configured-model"
    reasoning_effort: "high"
    mode: "read"
    prompt: "prompt.md"
    visible_paths: ["src/**", "docs/**"]
    timeout_seconds: 60
    memory_bytes: 268435456
    max_processes: 32
    skills: ["skills/example"]
    programs:
      python: "/usr/bin/python3"
    credentials:
      codex_auth_file_env: "DELEGATE_AUTH_FILE"
    mcp_servers:
      assets:
        program: "python"
        args: ["-m", "configured_server"]
        env:
          SERVICE_TOKEN: "PROJECT_SERVICE_TOKEN"
"""


def check(worker: Path, root: Path, config: str) -> subprocess.CompletedProcess[str]:
    directory = root / "config"
    directory.mkdir()
    (directory / "delegate.yaml").write_text(config)
    (directory / "prompt.md").write_text(
        "Perform the assigned task within the supplied contract.\n"
    )
    skill = directory / "skills/example"
    skill.mkdir(parents=True)
    (skill / "SKILL.md").write_text(
        "---\nname: example\ndescription: Example task guidance.\n---\n"
    )
    return subprocess.run(
        [str(worker), "delegate", "--root", str(root), "config-check", "config/delegate.yaml"],
        capture_output=True,
        text=True,
        check=False,
    )


@pytest.mark.parametrize("mode", ["read", "artifacts", "code"])
def test_profile_resources_resolve_from_config_without_loading_secrets(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, mode: str
) -> None:
    monkeypatch.delenv("DELEGATE_AUTH_FILE", raising=False)
    monkeypatch.delenv("PROJECT_SERVICE_TOKEN", raising=False)
    result = check(worker, tmp_path, CONFIG.replace('mode: "read"', f'mode: "{mode}"'))
    assert result.returncode == 0, result.stderr
    assert "1 profiles" in result.stdout


@pytest.mark.parametrize(
    "case",
    [
        ('frontend: "codex"', 'frontend: "unknown"', "unknown variant"),
        ('mode: "read"', 'mode: "host-write"', "unknown variant"),
        ('reasoning_effort: "high"', 'reasoning_effort: "invalid"', "reasoning effort"),
        ('prompt: "prompt.md"', 'prompt: "missing.md"', "resolve resource"),
        (
            'skills: ["skills/example"]',
            'skills: ["skills/example", "skills/example"]',
            "duplicate skill",
        ),
        ('program: "python"', 'program: "missing"', "unknown program"),
        ("timeout_seconds: 60", "timeout_seconds: 0", "timeout_seconds must be positive"),
        ("max_processes: 32", "max_processes: 0", "max_processes must be positive"),
        ("memory_bytes: 268435456", "memory_bytes: 0", "memory_bytes must be positive"),
        (
            'SERVICE_TOKEN: "PROJECT_SERVICE_TOKEN"',
            'SERVICE_TOKEN: "literal-secret-value"',
            "variable names",
        ),
        (
            'SERVICE_TOKEN: "PROJECT_SERVICE_TOKEN"',
            'HOME: "PROJECT_SERVICE_TOKEN"',
            "owned by the sandbox",
        ),
        (
            'codex_auth_file_env: "DELEGATE_AUTH_FILE"',
            "codex_auth_file_env: null",
            "credentials requires",
        ),
        ("schema_version: 1", "schema_version: 2", "unsupported delegation"),
        ("schema_version: 1", "schema_version: 1\nunknown: true", "unknown field"),
        ("schema_version: 1", "schema_version: 1\nvcs: unknown", "unknown variant"),
        ("schema_version: 1", "schema_version: 1\nvcs:\n  command: []", "external VCS command"),
        (
            "schema_version: 1",
            "schema_version: 1\nvcs:\n  command: [python3]\n  extra: true",
            "unknown field",
        ),
        ("timeout_seconds: 60", "timeout_seconds: '60'", "profiles.reader.timeout_seconds"),
        ("timeout_seconds: 60", "timeout_seconds: 60\n    timeout_seconds: 10", "duplicate"),
        ('visible_paths: ["src/**", "docs/**"]', 'visible_paths: ["["]', "glob"),
    ],
)
def test_invalid_profile_configuration(
    worker: Path, tmp_path: Path, case: tuple[str, str, str]
) -> None:
    old, new, message = case
    assert old in CONFIG
    result = check(worker, tmp_path, CONFIG.replace(old, new))
    assert result.returncode != 0
    assert message in result.stderr


HOOK = """    hooks:
      reminder:
        event: SessionStart
        program: python
        args: ["--version"]
        timeout_seconds: 10
"""


@pytest.mark.parametrize("event", ["SessionStart", "PreToolUse", "PostToolUse", "Stop"])
def test_supported_command_hooks_validate(worker: Path, tmp_path: Path, event: str) -> None:
    result = check(worker, tmp_path, CONFIG + HOOK.replace("SessionStart", event))
    assert result.returncode == 0, result.stderr


@pytest.mark.parametrize(
    "case",
    [
        ("SessionStart", "UnsupportedEvent", "unknown variant"),
        ("program: python", "program: missing", "hook reminder references unknown program"),
        (
            "event: SessionStart",
            'event: SessionStart\n        matcher: "["',
            "hook reminder matcher",
        ),
        (
            "event: SessionStart",
            'event: Stop\n        matcher: "*"',
            "Stop does not support matcher",
        ),
        ("timeout_seconds: 10", "timeout_seconds: 0", "hook reminder timeout_seconds"),
        ("timeout_seconds: 10", "timeout_seconds: '10'", "hooks.reminder.timeout_seconds"),
        ("event: SessionStart", "event: SessionStart\n        async: true", "unknown field"),
    ],
)
def test_hook_errors_are_actionable(
    worker: Path, tmp_path: Path, case: tuple[str, str, str]
) -> None:
    old, new, message = case
    result = check(worker, tmp_path, CONFIG + HOOK.replace(old, new))
    assert result.returncode != 0
    assert message in result.stderr


@pytest.mark.parametrize("matcher", ["*", "", "^startup$", "Bash|apply_patch"])
def test_hook_matchers_validate(worker: Path, tmp_path: Path, matcher: str) -> None:
    config = CONFIG + HOOK.replace(
        "event: SessionStart", f'event: SessionStart\n        matcher: "{matcher}"'
    )
    result = check(worker, tmp_path, config)
    assert result.returncode == 0, result.stderr
