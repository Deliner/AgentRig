# DECISION: D132
from pathlib import Path

import yaml

from Project.shared.testing.consumer import invoke, read_config

REPOSITORY = Path(__file__).resolve().parents[3]


def denied(argv: list[str]) -> set[str]:
    flags = argv[argv.index("--") + 1 :]
    assert set(flags[::2]) == {"-D"}, flags
    return set(flags[1::2])


def test_rust_scaffold_delivers_the_dev_gate_clippy_profile(worker: Path, tmp_path: Path) -> None:
    (tmp_path / "Cargo.toml").write_text('[package]\nname = "sample"\nversion = "0.1.0"\n')
    (tmp_path / "src").mkdir()
    (tmp_path / "src/main.rs").write_text("fn main() {}\n")
    init = invoke(worker, tmp_path, "init", "--language", "rust", "--source", "src")
    assert init.returncode == 0, init.stderr
    config = read_config(tmp_path)
    argv = config["commands"]["clippy"]["argv"]
    assert argv[:5] == ["cargo", "clippy", "--manifest-path", "Cargo.toml", "--"]
    gate = yaml.safe_load((REPOSITORY / ".agentrig/commands.yaml").read_text())
    assert denied(argv) == denied(gate["clippy"]["argv"])
    check = {check["id"]: check for check in config["checks"]}["clippy"]
    assert (check["kind"], check["command"]) == ("command", "clippy")


def test_python_scaffold_has_no_clippy(worker: Path, tmp_path: Path) -> None:
    assert invoke(worker, tmp_path, "init", "--language", "python").returncode == 0
    config = read_config(tmp_path)
    assert "clippy" not in config["commands"]
    assert all(check["id"] != "clippy" for check in config["checks"])
