// DECISION: D005
use super::{config::Context, process};
use anyhow::{Context as _, Result, ensure};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    process::Output,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Serialize, Deserialize)]
struct Record {
    timestamp: u64,
    command: String,
    argv: Vec<String>,
    duration_seconds: f64,
    exit_code: i32,
}
pub fn argv(context: &Context, name: &str, extra: &[String]) -> Result<Vec<String>> {
    let command = context
        .config
        .commands
        .get(name)
        .with_context(|| format!("unknown command {name}"))?;
    ensure!(
        extra.is_empty() || command.accepts_args,
        "command {name} does not accept arguments"
    );
    let argv: Vec<_> = command.argv.iter().chain(extra).cloned().collect();
    ensure!(!argv.is_empty(), "command {name} needs argv");
    Ok(argv)
}
pub fn run(context: &Context, name: &str, extra: &[String]) -> Result<i32> {
    let argv = argv(context, name, extra)?;
    match execute(context, name, argv, false) {
        Ok(output) => Ok(process::exit_code(&output)),
        Err(error) => {
            eprintln!("{error:#}");
            Ok(127)
        }
    }
}
pub fn execute(context: &Context, name: &str, argv: Vec<String>, capture: bool) -> Result<Output> {
    let spec = &context.config.commands[name];
    let cwd = context.path(&spec.cwd)?;
    let runtime = context.path(&context.config.paths.runtime)?;
    let mut job = discipline_worker::jobs::Job::create(&runtime, &context.root, name, &cwd)?;
    job.prepare(&argv)?;
    let started = Instant::now();
    let result = process::tracked(&cwd, &argv, (spec.read_only, capture), Some(&mut job));
    let code = result.as_ref().map(process::exit_code).unwrap_or(127);
    job.finish(
        code,
        result.as_ref().err().map(|error| format!("{error:#}")),
    )?;
    append(
        context,
        &Record {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            command: name.into(),
            argv,
            duration_seconds: started.elapsed().as_secs_f64(),
            exit_code: code,
        },
    )?;
    result
}
fn append(context: &Context, record: &Record) -> Result<()> {
    let directory = context.path(&context.config.paths.runtime)?;
    fs::create_dir_all(&directory)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(directory.join("commands.jsonl"))?;
    file.lock_exclusive()?;
    serde_json::to_writer(&mut file, record)?;
    writeln!(file)?;
    file.sync_all()?;
    Ok(())
}
pub fn report(context: &Context) -> Result<()> {
    let path = context
        .path(&context.config.paths.runtime)?
        .join("commands.jsonl");
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.into()),
    };
    let mut totals = BTreeMap::<String, (usize, usize, f64, f64)>::new();
    for line in source.lines() {
        let Ok(record) = serde_json::from_str::<Record>(line) else {
            continue;
        };
        let entry = totals.entry(record.command).or_default();
        entry.0 += 1;
        entry.1 += usize::from(record.exit_code != 0);
        entry.2 += record.duration_seconds;
        entry.3 = entry.3.max(record.duration_seconds);
    }
    println!("command calls failures total_s avg_s max_s");
    for (name, (calls, failed, total, max)) in totals {
        println!(
            "{name} {calls} {failed} {total:.3} {:.3} {max:.3}",
            total / calls as f64
        );
    }
    Ok(())
}
