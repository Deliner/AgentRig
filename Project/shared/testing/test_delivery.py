import json
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import pytest
import yaml

from Project.shared.testing.consumer import git, invoke, update_config


@dataclass(frozen=True)
class GitLayout:
    base: str
    prefix: str


@dataclass(frozen=True)
class ProjectPaths:
    source: str
    memory: str
    skills: str


@dataclass(frozen=True)
class Layout:
    language: str
    git: GitLayout
    paths: ProjectPaths

    def init_args(self) -> list[str]:
        return [
            "--language",
            self.language,
            "--source",
            self.paths.source,
            "--memory",
            self.paths.memory,
            "--skills",
            self.paths.skills,
            "--base",
            self.git.base,
            "--prefix",
            self.git.prefix,
        ]


@dataclass(frozen=True)
class Consumer:
    root: Path
    layout: Layout

    @property
    def python(self) -> bool:
        return self.layout.language == "python"

    @property
    def binary(self) -> Path:
        return self.root / ".agentrig/bin/agentrig"

    @property
    def source(self) -> Path:
        return self.root / self.layout.paths.source

    @property
    def test_path(self) -> Path:
        suffix = "test_sample.py" if self.python else "src/lib.rs"
        return self.source / suffix

    @property
    def oracle(self) -> str:
        return (
            f"{self.layout.paths.source}/test_sample.py::test_doubles"
            if self.python
            else "tests::doubles"
        )

    @property
    def marker(self) -> str:
        return "# INVARIANT: I001" if self.python else "// INVARIANT: I001"

    def install(self, worker: Path) -> None:
        git(self.root, "init", "-q", "-b", self.layout.git.base)
        git(self.root, "config", "user.name", "Test")
        git(self.root, "config", "user.email", "test@example.invalid")
        (self.root / ".gitignore").write_text("**/target/\n**/__pycache__/\n**/.pytest_cache/\n")
        git(self.root, "add", ".")
        git(self.root, "commit", "-qm", "base")
        git(self.root, "switch", "-c", self.layout.git.prefix + "bootstrap")
        result = invoke(worker, self.root, "init", *self.layout.init_args())
        assert result.returncode == 0, result.stderr
        assert self.binary.read_bytes() == worker.read_bytes()
        example = (
            Path(__file__).parent / "examples" / self.layout.language / self.layout.paths.source
        )
        shutil.copytree(
            example,
            self.source,
            ignore=shutil.ignore_patterns("target", "__pycache__", ".pytest_cache"),
        )
        self.bind_oracle()

    def bind_oracle(self) -> None:
        runner = "pytest" if self.python else "cargo"
        function = "test_doubles" if self.python else "doubles"
        link = "../" + self.test_path.relative_to(self.root).as_posix()
        update_config(
            self.root,
            oracles={"I001": {"check": "tests", "runner": runner, "target": self.oracle}},
        )
        memory = self.root / self.layout.paths.memory
        (memory / "Invariants").mkdir()
        index = memory / "Invariants.md"
        index.write_text(
            index.read_text()
            + f"| [I001](Invariants/001.md) | Doubles correctly | [{function}]({link}) |\n"
        )
        (memory / "Invariants/001.md").write_text(
            "# I001\n\n## Predicate\n\nDoubles correctly.\n\n## Oracle\n\nRun configured I001.\n"
        )

    def ready(self) -> None:
        for command in ["config-check", "doctor", "resume", "memory-check", "check"]:
            result = invoke(self.binary, self.root, command)
            assert result.returncode == 0, (command, result.stdout, result.stderr)

    def language_selection(self) -> None:
        config = self.root / ".agentrig/lint.yaml"
        before = config.read_text()
        assert invoke(self.binary, self.root, "lint").returncode == 0
        (self.source / "tool.sh").write_text("if true; then echo example; fi\n")
        config.write_text(
            before
            + f'\n- id: unsupported\n  kind: function-lines\n  target: file\n  include: ["{self.layout.paths.source}/**"]\n  extensions: [".sh"]\n  warning: 40\n  warning_skill: "{self.layout.paths.skills}/refactor-long-function/SKILL.md"\n  error_skill: "{self.layout.paths.skills}/refactor-long-function/SKILL.md"\n'
        )
        result = invoke(self.binary, self.root, "config-check")
        assert result.returncode == 2
        assert ".sh" in result.stderr and "handlers support" in result.stderr
        config.write_text(
            config.read_text().replace('extensions: [".sh"]', 'extensions: [".rs", ".py"]')
        )
        assert invoke(self.binary, self.root, "config-check").returncode == 0
        assert invoke(self.binary, self.root, "run", "test").returncode == 0
        self.just("config-check")
        self.just("run", "test")

    def just(self, *args: str) -> None:
        result = subprocess.run(
            ["just", *args], cwd=self.root, text=True, capture_output=True, check=False
        )
        assert result.returncode == 0, result.stdout + result.stderr

    def reject_marker_misbinding(self) -> None:
        valid = self.test_path.read_text()
        unrelated = (
            "# INVARIANT: I001\ndef unrelated():\n    pass\n"
            if self.python
            else "// INVARIANT: I001\nfn unrelated() {}\n"
        )
        other_scope = (
            "class Other:\n    # INVARIANT: I001\n    def test_doubles(self):\n        pass\n"
            if self.python
            else "mod other {\n    // INVARIANT: I001\n    fn doubles() {}\n}\n"
        )
        for prefix in [unrelated, other_scope]:
            self.test_path.write_text(prefix + valid.replace(self.marker, ""))
            result = invoke(self.binary, self.root, "memory-check")
            assert result.returncode == 2
            assert "marked oracle function" in result.stderr
        self.test_path.write_text(valid)
        if self.python:
            self.python_markers(valid)

    def python_markers(self, valid: str) -> None:
        self.test_path.write_text(valid + "\ndef test_doubles():\n    pass\n")
        result = invoke(self.binary, self.root, "memory-check")
        assert result.returncode == 2
        assert "ambiguous" in result.stderr
        decorator = '@pytest.mark.parametrize("unused", [1, 2])'
        parameterized = "import pytest\n" + valid.replace(
            "def test_doubles()", "def test_doubles(unused)"
        )
        for replacement in [self.marker + "\n" + decorator, decorator + "\n" + self.marker]:
            self.test_path.write_text(parameterized.replace(self.marker, replacement))
            result = invoke(self.binary, self.root, "memory-check")
            assert result.returncode == 0, result.stderr
        self.test_path.write_text(valid)

    def oracle_discovery(self) -> None:
        config = self.root / ".agentrig/oracles.yaml"
        valid_config = config.read_text()
        missing = self.oracle.replace("::", "::Missing::", 1)
        config.write_text(valid_config.replace(self.oracle, missing))
        result = invoke(self.binary, self.root, "memory-check")
        assert result.returncode == 2
        assert "marked oracle function" in result.stderr
        config.write_text(valid_config)
        valid = self.test_path.read_text()
        undiscoverable = (
            "__test__ = False\n" + valid if self.python else valid.replace("#[test]", "")
        )
        self.test_path.write_text(undiscoverable)
        result = invoke(self.binary, self.root, "memory-check")
        assert result.returncode == 2
        assert "test discovery failed" in result.stderr or "was not discovered" in result.stderr
        self.test_path.write_text(valid)
        records = json.loads(invoke(self.binary, self.root, "jobs").stdout)
        assert any("--collect-only" in row["argv"] or "--list" in row["argv"] for row in records)

    def hooks(self) -> None:
        hook = {
            "hook_event_name": "PreToolUse",
            "tool_name": "Write",
            "tool_input": {"file_path": f"{self.layout.paths.memory}/State.md"},
            "cwd": str(self.root),
        }
        result = invoke(self.binary, self.root, "hook", input=json.dumps(hook))
        assert result.returncode == 0
        assert f"{self.layout.paths.skills}/edit-state/SKILL.md" in result.stdout
        hook.update(tool_name="exec_command", tool_input={"cmd": "just run unknown"})
        assert "deny" in invoke(self.binary, self.root, "hook", input=json.dumps(hook)).stdout
        hook["tool_input"] = {"cmd": "echo bypass"}
        result = invoke(self.binary, self.root, "hook", input=json.dumps(hook))
        assert result.returncode == 0
        assert "deny" in result.stdout
        hook["tool_input"] = {"cmd": "just run test"}
        assert "deny" not in invoke(self.binary, self.root, "hook", input=json.dumps(hook)).stdout
        hook["tool_input"] = {"cmd": "just check --only lint"}
        first = invoke(self.binary, self.root, "hook", input=json.dumps(hook)).stdout
        assert "deny" in first and "edit-state/SKILL.md for VAC check entry" in first
        assert "deny" not in invoke(self.binary, self.root, "hook", input=json.dumps(hook)).stdout
        self.structure_route()

    def structure_route(self) -> None:
        source = self.layout.paths.source
        skill = f"{self.layout.paths.skills}/refactor-large-directory/SKILL.md"
        edit = {
            "hook_event_name": "PreToolUse",
            "tool_name": "Write",
            "tool_input": {"file_path": f"{source}/unit/first"},
            "cwd": str(self.root),
        }
        blocked = invoke(self.binary, self.root, "hook", input=json.dumps(edit)).stdout
        assert "deny" in blocked and skill in blocked
        edit["tool_input"] = {"file_path": f"{source}/unit/second"}
        assert "deny" not in invoke(self.binary, self.root, "hook", input=json.dumps(edit)).stdout

    def integrate(self) -> None:
        git(self.root, "add", ".")
        self.just("check", "--staged")
        git(self.root, "commit", "-qm", "attach scaffold")
        self.just("feature-merge")
        assert git(self.root, "branch", "--show-current").stdout.strip() == self.layout.git.base
        assert (
            git(
                self.root, "branch", "-D", self.layout.git.prefix + "bootstrap", expect="failure"
            ).returncode
            != 0
        )
        self.just("feature-start", "next")
        assert (
            git(self.root, "branch", "--show-current").stdout.strip()
            == self.layout.git.prefix + "next"
        )
        assert not (self.root / "Project/Cargo.toml").exists()
        assert invoke(self.binary, self.root, "init").returncode == 2


@pytest.mark.parametrize(
    "layout",
    [
        Layout(
            "python", GitLayout("trunk", "topic/"), ProjectPaths("application", "notes", "guides")
        ),
        Layout(
            "rust",
            GitLayout("release", "change/"),
            ProjectPaths("crates/engine", "knowledge", "policies"),
        ),
    ],
)
# INVARIANT: I002
def test_independent_project_delivery(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, layout: Layout
) -> None:
    monkeypatch.setenv("PATH", str(Path(sys.executable).parent) + os.pathsep + os.environ["PATH"])
    monkeypatch.delenv("CARGO_TARGET_DIR", raising=False)
    consumer = Consumer(tmp_path, layout)
    consumer.install(worker)
    consumer.ready()
    consumer.language_selection()
    consumer.reject_marker_misbinding()
    consumer.oracle_discovery()
    consumer.hooks()
    consumer.integrate()


def test_init_collision_preserves_files(worker: Path, tmp_path: Path) -> None:
    path = tmp_path / "justfile"
    path.write_text("user contents")
    result = invoke(worker, tmp_path, "init")
    assert result.returncode == 2
    assert "collision" in result.stderr
    assert list(tmp_path.iterdir()) == [path]
    assert path.read_text() == "user contents"


# DECISION: D053
WORKER_LOCAL_CHECKS = frozenset(
    {"build", "rustfmt", "clippy", "rust-test", "ruff-format", "ruff", "mypy", "typos", "vulture"}
)


def capabilities(service: Path, skills: Path, worker_local: frozenset[str]) -> dict[str, set[str]]:
    lint = yaml.safe_load((service / "lint.yaml").read_text())
    hooks = yaml.safe_load((service / "hooks.yaml").read_text())
    checks = yaml.safe_load((service / "checks.yaml").read_text())
    return {
        "rule-kinds": {rule["kind"] for rule in lint["rules"]},
        "skills": {found.parent.name for found in skills.rglob("SKILL.md")},
        "routes": {route["skill"].split("/")[-2] for route in hooks["routes"]},
        "checks": {check_capability(check) for check in checks} - worker_local,
    }


def check_capability(check: dict[str, Any]) -> str:
    command_kind = check["kind"] == "command"
    return str(check["command"] if command_kind else check["kind"])


def divergences(delivered: dict[str, set[str]], live: dict[str, set[str]]) -> list[str]:
    report: list[str] = []
    for dimension in sorted(delivered):
        only_delivered = delivered[dimension] - live[dimension]
        only_live = live[dimension] - delivered[dimension]
        if only_delivered:
            report.append(f"{dimension} only in init delivery: {sorted(only_delivered)}")
        if only_live:
            report.append(f"{dimension} only in repository: {sorted(only_live)}")
    return report


# INVARIANT: I020
def test_dogfooding_capabilities_stay_universal(worker: Path, tmp_path: Path) -> None:
    assert invoke(worker, tmp_path, "init", "--language", "python").returncode == 0
    repository = Path(__file__).resolve().parents[3]
    service = tmp_path / ".agentrig"
    delivered = capabilities(service, service / "skills", WORKER_LOCAL_CHECKS)
    live_service = repository / ".agentrig"
    live_skills = repository / "Project/shared/skills"
    live = capabilities(live_service, live_skills, WORKER_LOCAL_CHECKS)
    assert divergences(delivered, live) == [], divergences(delivered, live)
    exposed = capabilities(live_service, live_skills, frozenset())
    toolchain = divergences(delivered, exposed)
    assert any("build" in message for message in toolchain), toolchain
    for dimension in ("rule-kinds", "skills", "routes"):
        one_sided = {**live, dimension: live[dimension] | {"demo"}}
        report = divergences(delivered, one_sided)
        assert any(dimension in message and "demo" in message for message in report), report
