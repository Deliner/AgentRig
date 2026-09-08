from pathlib import Path

from tooling.worker.src.scaffold.testing.consumer import (
    invoke,
    update_config,
)


def declaration(worker: Path, root: Path, service: str = ".agentrig") -> Path:
    seed = root / "seed"
    seed.mkdir()
    result = invoke(worker, seed, "init", "--review", "true", "--service", service)
    assert result.returncode == 0, result.stderr
    consumer = root / "consumer"
    consumer.mkdir()
    (consumer / "agentrig.yaml").write_text(
        "# Consumer policy\n" + (seed / "agentrig.yaml").read_text()
    )
    (consumer / "src").mkdir()
    (consumer / "src/value.py").write_text("value = 1\n")
    return consumer


def delegated_project(worker: Path, root: Path, service: str = ".agentrig") -> Path:
    consumer = declaration(worker, root, service)
    config = consumer / "agentrig.yaml"
    update_config(config, capabilities={"delegation": {"config": "agents/profiles.yaml"}})
    agents = consumer / "agents"
    agents.mkdir()
    (agents / "prompt.md").write_text("Read the task and return its required JSON result.\n")
    (agents / "profiles.yaml").write_text(
        "schema_version: 1\nprofiles:\n  reader:\n    frontend: codex\n    model: consumer-model\n"
        "    reasoning_effort: high\n    mode: read\n    prompt: prompt.md\n"
        "    visible_paths: ['src/**']\n    timeout_seconds: 30\n"
        "    credentials:\n      env:\n        OPENAI_API_KEY: CONSUMER_KEY\n"
    )
    return consumer
