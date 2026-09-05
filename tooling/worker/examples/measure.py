"""Measure standalone consumer process latency without compiling worker."""

import argparse
import datetime
import json
import platform
import shutil
import statistics
import subprocess
import tempfile
import time
from pathlib import Path


def measure(
    binary: Path, root: Path, operation: tuple[str, dict[str, object]], samples: int
) -> tuple[float, float, float]:
    command, event = operation
    timings = []
    for _ in range(samples + 1):
        started = time.perf_counter()
        result = subprocess.run(
            [str(binary), command, "--root", str(root)],
            input=json.dumps(event),
            capture_output=True,
            text=True,
            check=False,
        )
        elapsed = (time.perf_counter() - started) * 1000
        if result.returncode:
            raise RuntimeError(result.stdout + result.stderr)
        timings.append(elapsed)
    warm = sorted(timings[1:])
    return timings[0], statistics.median(warm), warm[int((len(warm) - 1) * 0.95)]


def main() -> None:
    args = arguments()
    worker = args.worker.resolve()
    version = subprocess.run(
        [str(worker), "--version"], capture_output=True, text=True, check=True
    ).stdout.strip()
    rows = []
    for language, source in [("python", "application"), ("rust", "crates/engine")]:
        with tempfile.TemporaryDirectory(prefix=f"worker-latency-{language}-") as directory:
            root = Path(directory)
            shutil.copytree(Path(__file__).parent / language, root, dirs_exist_ok=True)
            subprocess.run(
                [
                    str(worker),
                    "init",
                    "--root",
                    str(root),
                    "--language",
                    language,
                    "--source",
                    source,
                ],
                capture_output=True,
                check=True,
            )
            binary = root / ".worker/bin/discipline-worker"
            events = operations()
            for label, command, event in events:
                first, median, p95 = measure(binary, root, (command, event), args.samples)
                rows.append(f"| {language} | {label} | {first:.3f} | {median:.3f} | {p95:.3f} |")
    source = report(version, args.samples, rows)
    args.output.write_text(source)
    print(source)


def arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--worker", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--samples", type=int, default=50)
    args = parser.parse_args()
    insufficient_samples = args.samples < 2
    if insufficient_samples:
        parser.error("--samples must be at least 2")
    return args


def operations() -> list[tuple[str, str, dict[str, object]]]:
    events: list[tuple[str, str, dict[str, object]]] = [
        (
            "SessionStart",
            "hook",
            {"hook_event_name": "SessionStart", "session_id": "latency"},
        ),
        (
            "PreToolUse edit",
            "hook",
            {
                "hook_event_name": "PreToolUse",
                "tool_name": "Write",
                "tool_input": {"file_path": "memory/State.md"},
            },
        ),
        ("small source lint", "lint", {}),
    ]
    return events


def report(version: str, samples: int, rows: list[str]) -> str:
    source = (
        f"""# Observed standalone latency

Measured {datetime.datetime.now(datetime.UTC).isoformat()} using {version} on {platform.system()} {platform.machine()}, {platform.release()}.

Each language uses a new consumer installation of its independent source example; its three operations share that installation. First is the first measured process for that operation; repeated results use {samples} fresh processes. This is a cold application process, not a flushed OS page cache. Compilation and installation are excluded. Timings include process startup, configuration loading, filesystem work and captured output. The edit event has no transcript; it measures file guidance, not large transcript scanning. Lint runs the five default rules on the small example source tree.

| Project | Operation | First ms | Repeated median ms | Repeated p95 ms |
| --- | --- | ---: | ---: | ---: |
"""
        + "\n".join(rows)
        + "\n\nThese local observations describe this workload, not a latency guarantee. No additional cache, daemon or parallel scheduler was introduced for this measurement. Reproduce with `python3 tooling/worker/examples/measure.py --worker /absolute/path/discipline-worker --output /path/to/report.md`.\n"
    )
    return source


running_script = __name__ == "__main__"
if running_script:
    main()
