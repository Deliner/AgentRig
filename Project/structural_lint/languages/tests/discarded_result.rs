use super::super::analyze;
use crate::structural_lint::rules::kind::Kind;
use crate::structural_lint::rules::reference::Reference;
use std::path::Path;

// DECISION: D090
fn discarded_findings(path: &str, source: &str) -> Vec<usize> {
    let analysis = analyze(Path::new(path), source).unwrap();
    assert_eq!(analysis.parse_error, None);
    let mut lines: Vec<_> = analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == Kind::Reference(Reference::DiscardedResult))
        .map(|item| item.line)
        .collect();
    lines.sort();
    lines
}

#[test]
fn rust_flags_discarded_result_statements_outside_tests() {
    let source = "fn run() {
    thing.ok();
    // DECISION: D001
    other.ok();
    let kept = thing.ok();
    saved = thing.ok();
    thing.ok()?;
    thing.run();
}
#[cfg(test)]
mod tests {
    fn check() {
        value.ok();
    }
}
";
    assert_eq!(discarded_findings("source.rs", source), [2]);
}

// DECISION: D099
#[test]
fn rust_flags_propagated_status_and_output_statements_that_drop_the_outcome() {
    let source = "fn run() -> io::Result<()> {
    Command::new(\"git\").status()?;
    // DECISION: D001
    Command::new(\"git\").output()?;
    command(root, args)
        .args([\"-c\"])
        .output()?;
    let status = Command::new(\"git\").status()?;
    Command::new(\"git\").status()?.success();
    check(Command::new(\"git\").output()?);
    writer.output(data)?;
    Command::new(\"git\").status();
    thing.ok()?;
    Ok(())
}
#[cfg(test)]
mod tests {
    fn check() {
        Command::new(\"git\").status()?;
    }
}
";
    assert_eq!(discarded_findings("source.rs", source), [2, 5]);
}

// DECISION: D114
#[test]
fn rust_flags_discarded_option_combinator_and_err_statements() {
    let source = "fn run(value: Option<u8>) -> Option<u8> {
    value.map(|item| store(item));
    value.and_then(|item| load(item));
    value.map_or(0, |item| send(item));
    value.map_or_else(|| 0, |item| send(item));
    outcome.err();
    // DECISION: D001
    value.map(|item| store(item));
    let kept = value.map(|item| store(item));
    saved = value.and_then(|item| load(item));
    check(value.map(|item| store(item)));
    value.map(|item| store(item))?;
    value.filter(|item| *item > 0);
    value.map(|item| item + 1)
}
#[cfg(test)]
mod tests {
    fn check() {
        value.map(|item| store(item));
    }
}
";
    assert_eq!(discarded_findings("source.rs", source), [2, 3, 4, 5, 6]);
}

// DECISION: D155
#[test]
fn rust_flags_dropped_spawn_handles_of_threads_tasks_and_children() {
    let source = "fn run() -> io::Result<()> {
    thread::spawn(move || work());
    std::thread::spawn(|| work()).join();
    tokio::spawn(async move { serve().await });
    let _ = Command::new(\"git\").spawn()?;
    Command::new(\"git\").spawn().unwrap();
    builder.spawn(task).expect(\"spawn\");
    // DECISION: D001
    thread::spawn(move || detached());
    let handle = thread::spawn(move || work());
    let _guard = thread::spawn(move || work());
    handles.push(thread::spawn(move || work()));
    spawner.spawned();
    let _ = start(task);
    Ok(())
}
#[cfg(test)]
mod tests {
    fn check() {
        thread::spawn(|| work());
    }
}
";
    assert_eq!(discarded_findings("source.rs", source), [2, 4, 5, 6, 7]);
}
