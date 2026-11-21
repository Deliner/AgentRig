use super::evaluation::evaluate;
use super::location::Location;
use crate::shared::console;
use crate::shared::diagnostics::{Diagnostic, Level};
use crate::shared::fs_with_context as fs;
use crate::shared::vcs::Repository;
use crate::structural_lint::lint_files::{Counts, Key, baseline_path, ceiling, committed, pairs};
use anyhow::{Context, Result};
use std::path::Path;

fn load(path: &Path) -> Result<Option<Counts>> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) if error.missing() => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let counts = crate::shared::strict_yaml::decode(&source)
        .with_context(|| format!("lint baseline {}", path.display()))?;
    Ok(Some(counts))
}

fn counts(diagnostics: &[Diagnostic]) -> Counts {
    let mut counts = Counts::new();
    for item in diagnostics.iter().filter(|item| item.level.blocking()) {
        *counts
            .entry(item.rule.clone())
            .or_default()
            .entry(item.path.clone())
            .or_default() += 1;
    }
    counts
}

// DECISION: D123
pub(super) fn ratchet(config: &Path, diagnostics: &mut [Diagnostic]) -> Result<()> {
    let path = baseline_path(config);
    let Some(baseline) = load(&path)? else {
        return Ok(());
    };
    let current = counts(diagnostics);
    for item in diagnostics.iter_mut().filter(|item| item.level.blocking()) {
        let rule = &item.rule;
        let path = &item.path;
        let key = Key { rule, path };
        let limit = key.allowed(&baseline);
        let count = key.allowed(&current);
        let accepted = count <= limit;
        if accepted {
            item.level = Level::Warning;
            item.message.push_str("; accepted by the lint baseline");
        } else {
            item.message.push_str(&format!(
                "; the lint baseline accepts {limit} of these {count} findings in this path"
            ));
        }
    }
    let lowered = pairs(&baseline)
        .filter(|pair| pair.key.allowed(&current) < pair.count)
        .count();
    let lowerable = lowered > 0;
    if lowerable {
        console::stderr(format!(
            "Lint findings fell below the baseline for {lowered} rule and path pairs; run lint-baseline to lower {}",
            path.display()
        ));
    }
    Ok(())
}

fn current(lint: Location<'_>) -> Result<Counts> {
    let config = lint.load()?;
    Ok(counts(&evaluate(lint.root, &config)?))
}

pub fn error_count(lint: Location<'_>) -> Result<u64> {
    Ok(pairs(&current(lint)?).map(|pair| pair.count).sum())
}

// DECISION: D123
// DECISION: D139
// DECISION: D214
pub fn record_baseline(lint: Location<'_>) -> Result<i32> {
    let current = current(lint)?;
    let path = baseline_path(lint.config);
    let findings = match load(&path)? {
        Some(baseline) => lowered(limit(lint, baseline)?, &current),
        None => current.clone(),
    };
    let exceeding = pairs(&current)
        .filter(|pair| pair.key.allowed(&findings) < pair.count)
        .count();
    let total: u64 = pairs(&findings).map(|pair| pair.count).sum();
    fs::write(&path, crate::shared::strict_yaml::encode(&findings)?)?;
    console::stdout(format!(
        "Recorded {total} accepted lint errors in {}",
        path.display()
    ));
    let unaccepted = exceeding > 0;
    if unaccepted {
        console::stdout(format!(
            "{exceeding} rule and path pairs exceed the baseline and were not raised; fix them, or ask the user to delete {} and to approve with just approve the baseline that lint-baseline then records",
            path.display()
        ));
    }
    Ok(0)
}

// DECISION: D139
// DECISION: D140
fn limit(lint: Location<'_>, working: Counts) -> Result<Counts> {
    let root = lint.root;
    let path = baseline_path(lint.config);
    let Some(repository) = Repository::discover(root)? else {
        return Ok(working);
    };
    let (Some(head), Ok(relative)) = (repository.head()?, path.strip_prefix(root)) else {
        return Ok(working);
    };
    let committed = committed(&repository, &head, relative)?;
    let renames = repository.renames(&head)?;
    Ok(ceiling(&working, committed.as_ref(), &renames))
}

// DECISION: D123
fn lowered(mut baseline: Counts, current: &Counts) -> Counts {
    for (rule, paths) in &mut baseline {
        for (path, count) in paths.iter_mut() {
            *count = Key { rule, path }.allowed(current).min(*count);
        }
        paths.retain(|_, count| *count > 0);
    }
    baseline.retain(|_, paths| !paths.is_empty());
    baseline
}
