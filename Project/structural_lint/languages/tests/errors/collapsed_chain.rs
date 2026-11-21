use super::silent_findings;

#[test]
fn rust_flags_read_dir_chains_that_drop_failed_entries() {
    let source = "use std::fs;
fn list(root: &Path) -> Result<Vec<PathBuf>> {
    let direct: Vec<_> = fs::read_dir(root)?.flatten().map(|entry| entry.path()).collect();
    let owned: Vec<_> = std::fs::read_dir(root)?.into_iter().flatten().collect();
    let method: Vec<_> = root.read_dir()?.filter_map(Result::ok).collect();
    let closure: Vec<_> = read_dir(root)?.filter_map(|entry| entry.ok()).collect();
    let bounded: Vec<_> = fs::read_dir(root)?
        .map_while(Result::ok)
        .collect();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
    }
    let kept = fs::read_dir(root)?.map(|entry| entry.map(|found| found.path()));
    let unrelated: Vec<_> = lines.iter().flatten().collect();
    Ok(direct)
}
#[test]
fn probe() -> Result<()> {
    let count = fs::read_dir(\".\")?.flatten().count();
    Ok(())
}
";
    assert_eq!(
        silent_findings("source.rs", source),
        [
            "3:flatten",
            "4:flatten",
            "5:filter_map",
            "6:filter_map",
            "7:map_while"
        ]
    );
}

// DECISION: D165
#[test]
fn rust_flags_filter_map_and_flat_map_that_keep_only_the_ok_values_of_any_chain() {
    let source =
        "fn load(items: &[&str], paths: &[PathBuf], text: &str, root: &Path) -> Result<()> {
    let parsed: Vec<u32> = items.iter().map(|item| item.parse()).filter_map(Result::ok).collect();
    let flat: Vec<u32> = items.iter().map(|item| item.parse()).flat_map(Result::ok).collect();
    let loaded: Vec<Config> = paths.iter().map(load).filter_map(|loaded| loaded.ok()).collect();
    let called: Vec<u32> = text.split(',').filter_map(|part| part.parse().ok()).collect();
    let numbers: Vec<u32> = text.lines().filter_map(|line| line.parse().ok()).collect();
    let pairs: Vec<u32> = table.iter().flat_map(|(_, value)| value.parse().ok()).collect();
    let entries: Vec<_> = fs::read_dir(root)?.flat_map(Result::ok).collect();
    Ok(())
}
";
    assert_eq!(
        silent_findings("source.rs", source),
        [
            "2:filter_map",
            "3:flat_map",
            "4:filter_map",
            "5:filter_map",
            "6:filter_map",
            "7:flat_map",
            "8:flat_map"
        ]
    );
}

// DECISION: D165
#[test]
fn rust_leaves_lines_spellings_to_clippy_and_keeps_options_errors_and_test_chains() {
    let source = "fn read(reader: impl BufRead, items: &[&str]) -> Vec<String> {
    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
    let closure: Vec<String> = reader.lines().flat_map(|line| line.ok()).collect();
    let options: Vec<u32> = items.iter().filter_map(|item| lookup(item)).collect();
    let errors: Vec<Error> = results.into_iter().filter_map(Result::err).collect();
    let chained: Vec<u32> = items.iter().filter_map(|item| item.parse().ok().map(double)).collect();
    let flattened: Vec<u32> = items.iter().map(|item| item.parse()).flatten().collect();
    let bounded: Vec<u32> = items.iter().map(|item| item.parse()).map_while(Result::ok).collect();
    lines
}
#[cfg(test)]
mod tests {
    fn probe(items: &[&str]) -> Vec<u32> {
        items.iter().filter_map(|item| item.parse().ok()).collect()
    }
}
";
    assert!(silent_findings("source.rs", source).is_empty());
}

// DECISION: D207
#[test]
fn rust_flags_ok_flatten_and_join_fallbacks_that_erase_the_error() {
    let source =
        "fn load(saved: &Value, handles: Vec<Handle>, workers: Vec<Handle>) -> Result<Seed> {
    let seed = saved::seed(integer(&field(saved, \"seed\")).ok().flatten())?;
    let duplicate = request.error().ok().flatten().is_some_and(|error| error.duplicate());
    let chunks: Vec<_> = handles.into_iter().flat_map(|h| h.join().unwrap_or_default()).collect();
    let sum = workers.into_iter().fold(0, |sum, w| sum + w.join().unwrap_or_default());
    let first = simulation.join().unwrap_or_default();
    let kept = worker.join().ok();
    let fallback = worker.join().unwrap_or(Vec::new());
    let computed = worker.join().unwrap_or_else(|_| Vec::new());
    Ok(seed)
}
";
    assert_eq!(
        silent_findings("source.rs", source),
        [
            "2:flatten",
            "3:flatten",
            "4:unwrap_or_default",
            "5:unwrap_or_default",
            "6:unwrap_or_default",
            "7:ok",
            "8:unwrap_or",
            "9:unwrap_or_else"
        ]
    );
}

// DECISION: D207
#[test]
fn rust_keeps_raised_argued_plain_and_test_join_and_ok_chains() {
    let source = "fn keep(worker: Handle, cache: Cache, value: Option<Option<u32>>) -> Result<()> {
    let joined = worker.join().unwrap();
    let expected = worker.join().expect(\"the worker finishes\");
    let mapped = worker.join().map_err(|_| anyhow!(\"the worker panicked\"))?;
    let keyed = cache.join(key).unwrap_or_default();
    let nested = value.flatten();
    let parsed = text.parse::<u32>().ok().map(double);
    Ok(())
}
#[cfg(test)]
mod tests {
    fn probe(worker: Handle) -> u32 {
        worker.join().unwrap_or_default()
    }
}
";
    assert!(silent_findings("source.rs", source).is_empty());
}
