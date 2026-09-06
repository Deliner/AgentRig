import json
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

import pytest
from support import git, invoke


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
        return self.root / ".worker/bin/discipline-worker"

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
            Path(__file__).parents[3]
            / "worker/examples"
            / self.layout.language
            / self.layout.paths.source
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
        with (self.root / "worker.toml").open("a") as stream:
            stream.write(
                f'\n[oracles.I001]\ncheck = "tests"\nrunner = "{runner}"\ntarget = "{self.oracle}"\n'
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
        config = self.root / ".worker/lint.toml"
        before = config.read_text()
        assert invoke(self.binary, self.root, "lint").returncode == 0
        (self.source / "tool.sh").write_text("if true; then echo example; fi\n")
        config.write_text(
            before
            + f'\n[[rules]]\nid = "unsupported"\nkind = "function-lines"\ntarget = "file"\ninclude = ["{self.layout.paths.source}/**"]\nextensions = [".sh"]\nwarning = 40\nwarning_skill = "{self.layout.paths.skills}/refactor-long-function/SKILL.md"\nerror_skill = "{self.layout.paths.skills}/refactor-long-function/SKILL.md"\n'
        )
        result = invoke(self.binary, self.root, "config-check")
        assert result.returncode == 2
        assert ".sh" in result.stderr and "handlers support" in result.stderr
        config.write_text(
            config.read_text().replace('extensions = [".sh"]', 'extensions = [".rs", ".py"]')
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
        config = self.root / "worker.toml"
        valid_config = config.read_text()
        missing = self.oracle.replace("::", "::Missing::", 1)
        config.write_text(
            valid_config.replace(f'target = "{self.oracle}"', f'target = "{missing}"')
        )
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
        for command in ["just run test", "just check --only lint"]:
            hook["tool_input"] = {"cmd": command}
            assert (
                "deny" not in invoke(self.binary, self.root, "hook", input=json.dumps(hook)).stdout
            )

    def integrate(self) -> None:
        git(self.root, "add", ".")
        self.just("check", "--staged")
        git(self.root, "commit", "-qm", "attach scaffold")
        self.just("feature-merge")
        assert git(self.root, "branch", "--show-current").stdout.strip() == self.layout.git.base
        assert (
            git(
                self.root, "branch", "-D", self.layout.git.prefix + "bootstrap", success=False
            ).returncode
            != 0
        )
        self.just("feature-start", "next")
        assert (
            git(self.root, "branch", "--show-current").stdout.strip()
            == self.layout.git.prefix + "next"
        )
        assert not (self.root / "tooling/worker/Cargo.toml").exists()
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
