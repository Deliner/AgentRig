import json
import os
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

import pytest
import yaml
from support import file_contents, git, invoke, update_config


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
        update_config(
            self.root / "agentrig.yaml",
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
        config = self.root / "agentrig.yaml"
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


def composed_consumer(worker: Path, directory: Path, language: str) -> Path:
    declaration = directory / f"declaration-{language}"
    service = f"rig-{language}"
    result = invoke(
        worker,
        declaration,
        "init",
        "--language",
        language,
        "--source",
        "application",
        "--service",
        service,
    )
    assert result.returncode == 0, result.stdout + result.stderr
    path = declaration / "agentrig.yaml"
    config = yaml.safe_load(path.read_text())
    config["packages"] = [{"path": "../shared.yaml", "id": "shared-command", "version": "1"}]
    config["overrides"] = ["/commands/probe/argv"]
    config["commands"]["probe"] = {"argv": ["python3", "-c", f"print({language!r})"]}
    path.write_text(yaml.safe_dump(config))
    target = directory / language
    target.mkdir()
    preview = invoke(worker, target, "setup", "--config", str(path), "--preview")
    assert preview.returncode == 0, preview.stdout + preview.stderr
    assert file_contents(target) == {}
    installed = invoke(worker, target, "setup", "--config", str(path))
    assert installed.returncode == 0, installed.stdout + installed.stderr
    example = Path(__file__).parents[3] / "worker/examples" / language
    example /= {"python": "application", "rust": "crates/engine"}[language]
    shutil.copytree(
        example,
        target / "application",
        ignore=shutil.ignore_patterns("target", "__pycache__", ".pytest_cache"),
    )
    receipt = json.loads((target / service / "composition.json").read_text())
    assert receipt["packages"][0]["id"] == "shared-command"
    assert receipt["provenance"]["/commands/probe/argv"] == str(path)
    shutil.rmtree(declaration)
    return target


def verify_composed_consumer(target: Path, standalone: Path) -> None:
    language = target.name
    binary = target / f"rig-{language}/bin/agentrig"
    assert not (target / "tooling/worker/Cargo.toml").exists()
    for command in ["config-check", "doctor", "check"]:
        result = invoke(binary, target, command)
        assert result.returncode == 0, result.stdout + result.stderr
    result = invoke(binary, target, "run", "probe")
    assert result.returncode == 0 and result.stdout.strip() == language
    before = file_contents(target)
    result = invoke(binary, target, "setup")
    assert result.returncode == 0, result.stdout + result.stderr
    assert file_contents(target) == before
    config = yaml.safe_load((target / "agentrig.yaml").read_text())
    policy = str(target / config["paths"]["lint"])
    python = language == "python"
    suffix = "test_sample.py" if python else "src/lib.rs"
    commands = [
        ["lint-rules"],
        ["lint-rule", "function-lines", "--example"],
        ["lint-explain", f"application/{suffix}", "--config", policy, "--json"],
        ["--config", policy, "--json"],
    ]
    for args in commands:
        result = subprocess.run(
            [standalone, *args], cwd=target, capture_output=True, text=True, check=False
        )
        assert result.returncode == 0, result.stdout + result.stderr
    assert file_contents(target) == before


def test_shared_package_prepares_independent_python_and_rust_consumers(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("PATH", str(Path(sys.executable).parent) + os.pathsep + os.environ["PATH"])
    shared = tmp_path / "shared.yaml"
    shared.write_text(
        yaml.safe_dump(
            {
                "schema_version": 1,
                "id": "shared-command",
                "version": "1",
                "configuration": {
                    "commands": {"probe": {"argv": ["python3", "-c", "print('shared')"]}}
                },
            }
        )
    )
    consumers = [composed_consumer(worker, tmp_path, language) for language in ["python", "rust"]]
    shared.unlink()
    standalone = tmp_path / "agentrig-lint"
    shutil.copy2(worker.with_name("agentrig-lint"), standalone)
    for target in consumers:
        verify_composed_consumer(target, standalone)


def installed_hg_consumer(worker: Path, root: Path, vcs: str) -> Path:
    from test_git import hg, initialize_mercurial

    hg(root, "init")
    hg(root, "branch", "trunk")
    (root / "README.md").write_text("Independent consumer\n")
    (root / ".hgignore").write_text("syntax: glob\n**/__pycache__/**\n**/.pytest_cache/**\n")
    hg(root, "add", "README.md", ".hgignore")
    hg(root, "commit", "-m", "base", "-u", "Test")
    hg(root, "branch", "task/bootstrap")
    initialize_mercurial(worker, root)
    private = vcs == "private"
    if private:
        adapter = root / "vcs_adapter.py"
        shutil.copy2(Path(__file__).parents[3] / "worker/examples/external_vcs.py", adapter)
        update_config(
            root / "agentrig.yaml", vcs={"backend": {"command": ["python3", "-B", str(adapter)]}}
        )
    assert invoke(worker, root, "setup").returncode == 0
    with (root / ".hg/hgrc").open("a") as settings:
        settings.write("\n[ui]\nusername = Test\n")
    source = root / "src/test_sample.py"
    source.parent.mkdir()
    source.write_text("def test_value():\n    assert 1 == 1\n")
    add_delivery_probe(root)
    hg(root, "add")
    hg(root, "commit", "-m", "install environment")
    binary = root / "rig space/bin/agentrig"
    assert binary.read_bytes() == worker.read_bytes()
    assert not (root / "tooling/worker/Cargo.toml").exists()
    return binary


def add_delivery_probe(root: Path) -> None:
    path = root / "agentrig.yaml"
    config = yaml.safe_load(path.read_text())
    config["commands"]["delivery-probe"] = {
        "argv": ["python3", "-c", "import os,sys; sys.exit(bool(os.environ.get('BLOCK_DELIVERY')))"]
    }
    config["checks"].append(
        {
            "id": "delivery-probe",
            "kind": "command",
            "command": "delivery-probe",
            "skill": "rig space/skills/repair/SKILL.md",
        }
    )
    path.write_text(yaml.safe_dump(config))


def consumer_just(root: Path, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["just", *args], cwd=root, text=True, capture_output=True, check=False)


@pytest.mark.parametrize("vcs", ["hg", "private"])
def test_installed_mercurial_delivery_recovers_failed_integration(
    worker: Path, tmp_path: Path, monkeypatch: pytest.MonkeyPatch, vcs: str
) -> None:
    from test_git import hg

    binary = installed_hg_consumer(worker, tmp_path, vcs)
    bootstrap = consumer_just(tmp_path, "feature-merge")
    assert bootstrap.returncode == 0, (
        bootstrap.stdout + bootstrap.stderr + invoke(binary, tmp_path, "resume").stdout
    )
    assert hg(tmp_path, "branch") == "trunk"
    base = hg(tmp_path, "log", "-r", ".", "-T", "{node}")
    assert consumer_just(tmp_path, "feature-start", "product").returncode == 0
    source = tmp_path / "src/test_sample.py"
    source.write_text("def test_value():\n    assert 2 == 2\n")
    hg(tmp_path, "commit", "-m", "product")
    candidate = hg(tmp_path, "log", "-r", ".", "-T", "{node}")
    monkeypatch.setenv("BLOCK_DELIVERY", "yes")
    failed = consumer_just(tmp_path, "feature-merge")
    assert failed.returncode != 0 and "delivery-probe" in failed.stderr
    recovery = json.loads(invoke(binary, tmp_path, "resume").stdout)
    assert recovery["vcs"]["merge_in_progress"]
    assert hg(tmp_path, "log", "-r", ".", "-T", "{node}") == base
    monkeypatch.delenv("BLOCK_DELIVERY")
    merged = consumer_just(tmp_path, "feature-merge")
    assert merged.returncode == 0, merged.stdout + merged.stderr
    assert hg(tmp_path, "log", "-r", ".", "-T", "{p1node} {p2node}").split() == [base, candidate]
    assert hg(tmp_path, "log", "-r", candidate, "-T", "{branch}") == "task/product"
    assert hg(tmp_path, "branch") == "trunk" and hg(tmp_path, "status") == ""
    assert invoke(binary, tmp_path, "doctor").returncode == 0
    assert consumer_just(tmp_path, "feature-start", "next").returncode == 0
