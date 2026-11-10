# DECISION: D132
# DECISION: D154
from pathlib import Path

import pytest
import yaml

from Project.shared.testing.consumer import file_contents, invoke, read_config

REPOSITORY = Path(__file__).resolve().parents[3]
THRESHOLDS = (REPOSITORY / "Project/clippy.toml").read_text()
TEAM = '# team\nmax-struct-bools = 1\nmsrv = "1.80"\n\n[[disallowed-methods]]\npath = "std::env::var"\n'


def denied(argv: list[str]) -> set[str]:
    flags = argv[argv.index("--") + 1 :]
    assert set(flags[::2]) == {"-D"}, flags
    return set(flags[1::2])


def rust_project(root: Path, crate: str = ".") -> None:
    (root / crate / "src").mkdir(parents=True)
    (root / crate / "Cargo.toml").write_text('[package]\nname = "sample"\nversion = "0.1.0"\n')
    (root / crate / "src/main.rs").write_text("fn main() {}\n")


def clippy_files(root: Path) -> list[str]:
    return sorted(path.relative_to(root).as_posix() for path in root.rglob("*clippy.toml"))


def test_rust_scaffold_delivers_the_dev_gate_clippy_profile(worker: Path, tmp_path: Path) -> None:
    rust_project(tmp_path)
    init = invoke(worker, tmp_path, "init", "--language", "rust", "--source", "src")
    assert init.returncode == 0, init.stderr
    config = read_config(tmp_path)
    argv = config["commands"]["clippy"]["argv"]
    assert argv[:5] == ["cargo", "clippy", "--manifest-path", "Cargo.toml", "--"]
    gate = yaml.safe_load((REPOSITORY / ".agentrig/commands.yaml").read_text())
    assert denied(argv) == denied(gate["clippy"]["argv"])
    check = {check["id"]: check for check in config["checks"]}["clippy"]
    assert (check["kind"], check["command"]) == ("command", "clippy")
    assert (tmp_path / "clippy.toml").read_text() == THRESHOLDS


@pytest.mark.parametrize("name", ["Project/clippy.toml", "Project/.clippy.toml", "clippy.toml"])
def test_rust_init_adds_missing_thresholds_to_the_effective_clippy_file(
    worker: Path, tmp_path: Path, name: str
) -> None:
    rust_project(tmp_path, "Project")
    (tmp_path / name).write_text(TEAM)
    init = invoke(worker, tmp_path, "init", "--language", "rust", "--source", "Project")
    assert init.returncode == 0, init.stderr
    assert (tmp_path / name).read_text() == "avoid-breaking-exported-api = false\n" + TEAM
    assert clippy_files(tmp_path) == [name]


def test_rust_init_completes_the_file_clippy_reads_first(worker: Path, tmp_path: Path) -> None:
    rust_project(tmp_path)
    (tmp_path / ".clippy.toml").write_text(TEAM)
    (tmp_path / "clippy.toml").write_text("max-struct-bools = 3\n")
    init = invoke(worker, tmp_path, "init", "--language", "rust", "--source", "src")
    assert init.returncode == 0, init.stderr
    assert (tmp_path / ".clippy.toml").read_text() == "avoid-breaking-exported-api = false\n" + TEAM
    assert (tmp_path / "clippy.toml").read_text() == "max-struct-bools = 3\n"


def test_rust_init_refuses_a_conflicting_clippy_threshold(worker: Path, tmp_path: Path) -> None:
    rust_project(tmp_path)
    (tmp_path / "clippy.toml").write_text("max-struct-bools = 3\n")
    before = file_contents(tmp_path)
    init = invoke(worker, tmp_path, "init", "--language", "rust", "--source", "src")
    assert init.returncode != 0
    assert "max-struct-bools = 3" in init.stderr
    assert file_contents(tmp_path) == before


def test_python_scaffold_has_no_clippy(worker: Path, tmp_path: Path) -> None:
    assert invoke(worker, tmp_path, "init", "--language", "python").returncode == 0
    config = read_config(tmp_path)
    assert "clippy" not in config["commands"]
    assert all(check["id"] != "clippy" for check in config["checks"])
    assert clippy_files(tmp_path) == []
