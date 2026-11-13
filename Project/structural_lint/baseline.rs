use super::Location;
use crate::shared::console;
use crate::shared::diagnostics::actionable_findings::{Diagnostic, Level};
use crate::shared::fs_with_context as fs;
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

#[derive(Clone, Copy)]
struct Key<'a> {
    rule: &'a str,
    path: &'a str,
}

impl Key<'_> {
    fn allowed(self, counts: &Counts) -> u64 {
        counts
            .get(self.rule)
            .and_then(|paths| paths.get(self.path))
            .copied()
            .unwrap_or_default()
    }
}

struct Pair<'a> {
    key: Key<'a>,
    count: u64,
}

fn pairs(counts: &Counts) -> impl Iterator<Item = Pair<'_>> {
    counts.iter().flat_map(|(rule, paths)| {
        paths.iter().map(move |(path, count)| Pair {
            key: Key { rule, path },
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
    Ok(counts(&super::evaluation::evaluate(lint.root, &config)?))
}

pub fn errors(lint: Location<'_>) -> Result<u64> {
    Ok(pairs(&current(lint)?).map(|pair| pair.count).sum())
}

// DECISION: D123
// DECISION: D139
pub fn record(lint: Location<'_>) -> Result<i32> {
    let current = current(lint)?;
    let path = location(lint.config);
    let findings = match load(&path)? {
        Some(baseline) => lowered(&limit(lint, baseline)?, &current),
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
            "{exceeding} rule and path pairs exceed the baseline and were not raised; fix them, or delete {} and record again to accept them",
            path.display()
        ));
    }
    Ok(0)
}

// DECISION: D139
// DECISION: D140
fn limit(lint: Location<'_>, working: Counts) -> Result<Counts> {
    let root = lint.root;
    let path = location(lint.config);
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
    let counts = crate::shared::strict_yaml::decode(&source)
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
        let path = renames
            .get(pair.key.path)
            .map_or(pair.key.path, String::as_str);
        let count = ceiling
            .entry(pair.key.rule.to_owned())
            .or_default()
            .entry(path.to_owned())
            .or_default();
        *count = (*count).max(pair.count);
    }
    ceiling
}

// DECISION: D123
fn lowered(baseline: &Counts, current: &Counts) -> Counts {
    let mut findings = Counts::new();
    for pair in pairs(baseline) {
        let count = pair.key.allowed(current).min(pair.count);
        let kept = count > 0;
        if kept {
            findings
                .entry(pair.key.rule.to_owned())
                .or_default()
                .insert(pair.key.path.to_owned(), count);
        }
    }
    findings
}
