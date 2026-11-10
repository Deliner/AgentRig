use crate::shared::console;
use crate::shared::diagnostics::lint::{Diagnostic, Level};
use crate::shared::fs;
use crate::shared::vcs::Repository;
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// DECISION: D123
type Counts = BTreeMap<String, BTreeMap<String, u64>>;

// DECISION: D123
pub fn location(config: &Path) -> PathBuf {
    let stem = config.file_stem().unwrap_or_default().to_string_lossy();
    config.with_file_name(format!("{stem}-baseline.yaml"))
}

fn load(path: &Path) -> Result<Option<Counts>> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) if error.missing() => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let counts = crate::shared::yaml::decode(&source)
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

fn allowed(counts: &Counts, rule: &str, path: &str) -> u64 {
    counts
        .get(rule)
        .and_then(|paths| paths.get(path))
        .copied()
        .unwrap_or_default()
}

struct Pair<'a> {
    rule: &'a str,
    path: &'a str,
    count: u64,
}

fn pairs(counts: &Counts) -> impl Iterator<Item = Pair<'_>> {
    counts.iter().flat_map(|(rule, paths)| {
        paths.iter().map(move |(path, count)| Pair {
            rule,
            path,
            count: *count,
        })
    })
}

// DECISION: D123
pub(super) fn ratchet(config: &Path, diagnostics: &mut [Diagnostic]) -> Result<()> {
    let path = location(config);
    let Some(baseline) = load(&path)? else {
        return Ok(());
    };
    let current = counts(diagnostics);
    for item in diagnostics.iter_mut().filter(|item| item.level.blocking()) {
        let limit = allowed(&baseline, &item.rule, &item.path);
        let count = allowed(&current, &item.rule, &item.path);
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
        .filter(|pair| allowed(&current, pair.rule, pair.path) < pair.count)
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

fn current(root: &Path, config: &Path) -> Result<Counts> {
    let config = super::config::load(root, config)?;
    Ok(counts(&super::evaluation::evaluate(root, &config)?))
}

pub fn errors(root: &Path, config: &Path) -> Result<u64> {
    Ok(pairs(&current(root, config)?).map(|pair| pair.count).sum())
}

// DECISION: D123
// DECISION: D139
pub fn record(root: &Path, config: &Path) -> Result<i32> {
    let current = current(root, config)?;
    let path = location(config);
    let findings = match load(&path)? {
        Some(baseline) => lowered(&limit(root, &path, baseline)?, &current),
        None => current.clone(),
    };
    let exceeding = pairs(&current)
        .filter(|pair| allowed(&findings, pair.rule, pair.path) < pair.count)
        .count();
    let total: u64 = pairs(&findings).map(|pair| pair.count).sum();
    fs::write(&path, crate::shared::yaml::encode(&findings)?)?;
    console::stdout(format!(
        "Recorded {total} accepted lint errors in {}",
        path.display()
    ));
    let unaccepted = exceeding > 0;
    if unaccepted {
        console::stdout(format!(
            "{exceeding} rule and path pairs exceed the baseline and were not raised; fix them, or delete {} and record again to accept them",
            path.display()
        ));
    }
    Ok(0)
}

// DECISION: D139
// DECISION: D140
fn limit(root: &Path, path: &Path, working: Counts) -> Result<Counts> {
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

fn committed(repository: &Repository<'_>, head: &str, relative: &Path) -> Result<Option<Counts>> {
    let name = relative.to_string_lossy();
    let listed = repository.run(&["ls-tree", "--name-only", head, "--", &name])?;
    let uncommitted = listed.is_empty();
    if uncommitted {
        return Ok(None);
    }
    let source = String::from_utf8(repository.read(head, relative)?)?;
    let counts = crate::shared::yaml::decode(&source)
        .with_context(|| format!("committed lint baseline {name}"))?;
    Ok(Some(counts))
}

fn ceiling(
    working: &Counts,
    committed: Option<&Counts>,
    renames: &BTreeMap<String, String>,
) -> Counts {
    let mut ceiling = Counts::new();
    for pair in pairs(working).chain(committed.into_iter().flat_map(pairs)) {
        let path = renames.get(pair.path).map_or(pair.path, String::as_str);
        let count = ceiling
            .entry(pair.rule.to_owned())
            .or_default()
            .entry(path.to_owned())
            .or_default();
        *count = (*count).max(pair.count);
    }
    ceiling
}

fn lowered(baseline: &Counts, current: &Counts) -> Counts {
    let mut findings = Counts::new();
    for pair in pairs(baseline) {
        let count = allowed(current, pair.rule, pair.path).min(pair.count);
        let kept = count > 0;
        if kept {
            findings
                .entry(pair.rule.to_owned())
                .or_default()
                .insert(pair.path.to_owned(), count);
        }
    }
    findings
}
