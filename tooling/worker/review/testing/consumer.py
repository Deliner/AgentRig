import json
from pathlib import Path


def resources(root: Path) -> None:
    (root / "config.yaml").write_text(
        "schema_version: 1\nrunner:\n  runtime_root: runtime\n  report_root: reports\n"
        "  isolation: bubblewrap\n  parallelism: 1\n  timeout_seconds: 5\n  format_attempts: 2\n"
        "reviewers:\n  critic:\n    model: test\n    reasoning_effort: high\n    prompt: prompt.md\n"
        "tools:\n  review_code:\n    description: Review\n    reviewers: [critic]\n"
        "    project_config: project.yaml\n"
    )
    (root / "project.yaml").write_text(
        "schema_version: 1\nrepository:\n  visible_paths: ['src/**']\n  contract_paths: []\n"
        "review:\n  contract: contract.json\n"
    )
    requirement = dict(id="C-1", text="Value is one", reviewers=["critic"], allow_na=False)
    (root / "contract.json").write_text(
        json.dumps(dict(schema_version=1, requirements=[requirement]))
    )
    (root / "prompt.md").write_text("Check the assigned requirement.")
    (root / "auth").mkdir()
    (root / "auth/auth.json").write_text("{}")
    source = Path(__file__).with_name("critic.py")
    (root / "codex").write_bytes(source.read_bytes())
    (root / "codex").chmod(0o755)
    (root / "codex-code-mode-host").write_text("unused")
