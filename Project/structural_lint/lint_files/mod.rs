mod rules;

use crate::shared::vcs::Repository;
use anyhow::{Context, Result};
pub use rules::only_adds_rules;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

// DECISION: D123
pub(super) type Counts = BTreeMap<String, BTreeMap<String, u64>>;

// DECISION: D123
pub fn baseline_path(config: &Path) -> PathBuf {
    let stem = config.file_stem().unwrap_or_default().to_string_lossy();
    config.with_file_name(format!("{stem}-baseline.yaml"))
}

#[derive(Clone, Copy)]
pub(super) struct Key<'a> {
    pub(super) rule: &'a str,
    pub(super) path: &'a str,
}

impl Key<'_> {
    pub(super) fn allowed(self, counts: &Counts) -> u64 {
        counts
            .get(self.rule)
            .and_then(|paths| paths.get(self.path))
            .copied()
            .unwrap_or_default()
    }
}

pub(super) struct Pair<'a> {
    pub(super) key: Key<'a>,
    pub(super) count: u64,
}

pub(super) fn pairs(counts: &Counts) -> impl Iterator<Item = Pair<'_>> {
    counts.iter().flat_map(|(rule, paths)| {
        paths.iter().map(move |(path, count)| Pair {
            key: Key { rule, path },
            count: *count,
        })
    })
}

pub(super) fn committed(
    repository: &Repository<'_>,
    head: &str,
    relative: &Path,
) -> Result<Option<Counts>> {
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

pub(super) fn ceiling(
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

// DECISION: D214
pub fn only_lowers_counts(repository: &Repository<'_>, relative: &Path) -> Result<bool> {
    let source = String::from_utf8(repository.read("", relative)?)?;
    let staged: Counts = crate::shared::strict_yaml::decode(&source)
        .with_context(|| format!("staged lint baseline {}", relative.display()))?;
    let limit = match repository.head()? {
        Some(head) => {
            let committed = committed(repository, &head, relative)?;
            ceiling(
                &Counts::new(),
                committed.as_ref(),
                &repository.renames(&head)?,
            )
        }
        None => Counts::new(),
    };
    Ok(pairs(&staged).all(|pair| pair.count <= pair.key.allowed(&limit)))
}
