use anyhow::{Context, Result, bail, ensure};
use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};

const USAGE: &str = "usage: measure --worker ABSOLUTE_AGENTRIG [--samples N]";
const CONSUMERS: [Consumer; 2] = [
    Consumer {
        language: "python",
        source: "application",
    },
    Consumer {
        language: "rust",
        source: "crates/engine",
    },
];
const OPERATIONS: [Operation; 3] = [
    Operation {
        label: "SessionStart",
        command: "hook",
        event: r#"{"hook_event_name": "SessionStart", "session_id": "latency"}"#,
    },
    Operation {
        label: "PreToolUse edit",
        command: "hook",
        event: r#"{"hook_event_name": "PreToolUse", "tool_name": "Write", "tool_input": {"file_path": "memory/State.md"}}"#,
    },
    Operation {
        label: "small source lint",
        command: "lint",
        event: "{}",
    },
];

struct Consumer {
    language: &'static str,
    source: &'static str,
}

struct Operation {
    label: &'static str,
    command: &'static str,
    event: &'static str,
}

struct Latency {
    first: Duration,
    median: Duration,
    p95: Duration,
}

// DECISION: D159
fn main() -> Result<()> {
    let (worker, samples) = arguments()?;
    let version = text(launch(Command::new(&worker).arg("--version"), "")?);
    let mut rows = Vec::new();
    for consumer in &CONSUMERS {
        rows.extend(consumer_rows(&worker, consumer, samples)?);
    }
    let measured = text(launch(
        Command::new("date").arg("-u").arg("+%Y-%m-%dT%H:%M:%SZ"),
        "",
    )?);
    let system = text(launch(Command::new("uname").arg("-srm"), "")?);
    let report = report(
        &format!("{measured} using {version} on {system}"),
        samples,
        &rows,
    );
    write!(std::io::stdout().lock(), "{report}")?;
    Ok(())
}

fn arguments() -> Result<(PathBuf, usize)> {
    let mut worker = None;
    let mut samples = 50;
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .with_context(|| format!("{flag} needs a value; {USAGE}"))?;
        match flag.as_str() {
            "--worker" => worker = Some(PathBuf::from(value)),
            "--samples" => samples = value.parse().context("--samples must be a number")?,
            _ => bail!("unknown argument {flag}; {USAGE}"),
        }
    }
    let worker = worker.context(USAGE)?;
    ensure!(worker.is_absolute(), "--worker must be an absolute path");
    ensure!(samples >= 2, "--samples must be at least 2");
    Ok((worker, samples))
}

fn consumer_rows(worker: &Path, consumer: &Consumer, samples: usize) -> Result<Vec<String>> {
    let Consumer { language, source } = *consumer;
    let directory = tempfile::Builder::new()
        .prefix(&format!("worker-latency-{language}-"))
        .tempdir()?;
    let root = directory.path();
    let example = Path::new(env!("CARGO_MANIFEST_DIR")).join("shared/testing/examples");
    let mut copy = Command::new("cp");
    copy.arg("-R")
        .arg(example.join(language).join("."))
        .arg(root);
    launch(&mut copy, "")?;
    let init = ["init", "--language", language, "--source", source, "--root"];
    launch(Command::new(worker).args(init).arg(root), "")?;
    let mut rows = Vec::new();
    for operation in &OPERATIONS {
        let latency = measure(root, operation, samples)?;
        let [first, median, p95] = [latency.first, latency.median, latency.p95].map(milliseconds);
        let label = operation.label;
        rows.push(format!(
            "| {language} | {label} | {first:.3} | {median:.3} | {p95:.3} |"
        ));
    }
    Ok(rows)
}

fn measure(root: &Path, operation: &Operation, samples: usize) -> Result<Latency> {
    let binary = root.join(".agentrig/bin/agentrig");
    let mut timings = Vec::with_capacity(samples + 1);
    for _ in 0..=samples {
        let mut command = Command::new(&binary);
        command.arg(operation.command).arg("--root").arg(root);
        let started = Instant::now();
        launch(&mut command, operation.event)?;
        timings.push(started.elapsed());
    }
    let mut warm = timings.split_off(1);
    warm.sort();
    let middle = warm.len() / 2;
    let odd = warm.len() % 2 == 1;
    let median = if odd {
        warm[middle]
    } else {
        (warm[middle - 1] + warm[middle]) / 2
    };
    let p95 = warm[warm.len().saturating_sub(1) * 95 / 100];
    Ok(Latency {
        first: timings[0],
        median,
        p95,
    })
}

fn launch(command: &mut Command, input: &str) -> Result<Output> {
    let piped = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = piped
        .spawn()
        .with_context(|| format!("cannot start {command:?}"))?;
    child
        .stdin
        .take()
        .context("stdin is not piped")?
        .write_all(input.as_bytes())?;
    let output = child.wait_with_output()?;
    let failed = !output.status.success();
    if failed {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "{command:?} failed with {}:\n{stdout}{stderr}",
            output.status
        );
    }
    Ok(output)
}

fn text(output: Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn milliseconds(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1000.0
}

fn report(measurement: &str, samples: usize, rows: &[String]) -> String {
    let rows = rows.join("\n");
    format!(
        "# Observed standalone latency

Measured {measurement}.

Each language uses a new consumer installation of its independent source example; its three operations share that \
installation. First is the first measured process for that operation; repeated results use {samples} fresh processes. \
This is a cold application process, not a flushed OS page cache. Compilation and installation are excluded. Timings \
include process startup, configuration loading, filesystem work and captured output. The edit event has no transcript; \
it measures file guidance, not large transcript scanning. Lint runs the five default rules on the small example source \
tree.

| Project | Operation | First ms | Repeated median ms | Repeated p95 ms |
| --- | --- | ---: | ---: | ---: |
{rows}

These local observations describe this workload, not a latency guarantee. No additional cache, daemon or parallel \
scheduler was introduced for this measurement. Reproduce with `cargo run --manifest-path Project/Cargo.toml --example \
measure -- --worker /absolute/path/agentrig > report.md`.
"
    )
}
