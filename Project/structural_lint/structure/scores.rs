use super::corpus::{Corpus, Counts, Source};
use super::units::{Unit, Units, Vector, name, real};
use super::words::split;
use std::path::{Path, PathBuf};

const KEY_TERMS: usize = 3;
const COVERED: f64 = 0.4;
const CLUSTERS: usize = 3;
const NAME_WORDS: usize = 3;
const ENTRIES: usize = 10;
const SIMILAR: f64 = 0.5;

pub struct Finding {
    pub score: f64,
    pub kind: &'static str,
    pub path: String,
    pub detail: String,
}

struct Problem {
    value: f64,
    kind: &'static str,
    detail: String,
}

pub fn measure(corpus: &Corpus) -> Vec<Finding> {
    let units = Units::new(corpus);
    let mut findings: Vec<Finding> = units
        .all()
        .iter()
        .map(|unit| finding(&units, unit))
        .collect();
    findings.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.path.cmp(&b.path))
    });
    findings
}

fn finding(units: &Units<'_>, unit: &Unit<'_>) -> Finding {
    let vector = units.vector(units.content(unit));
    let described = units.described(unit);
    let problems = [
        unnamed(units, unit, &described),
        hidden(units, unit),
        long(unit),
        crowded(units, unit),
        misplaced(units, unit, &vector),
        duplicate(units, unit, &vector),
    ];
    let worst = problems.into_iter().reduce(|worst, next| {
        let worse = next.value > worst.value;
        if worse { next } else { worst }
    });
    let shown = worst.filter(|problem| (problem.value * 100.0).round() > 0.0);
    let undescribed = described.is_empty();
    let quiet = Problem {
        value: 0.0,
        kind: if undescribed { "undescribed" } else { "ok" },
        detail: String::new(),
    };
    let problem = shown.unwrap_or(quiet);
    Finding {
        score: (problem.value * 100.0).round(),
        kind: problem.kind,
        path: label(unit),
        detail: problem.detail,
    }
}

fn label(unit: &Unit<'_>) -> String {
    let root = unit.path.as_os_str().is_empty();
    match (root, unit.directory) {
        (true, _) => ".".to_owned(),
        (false, true) => format!("{}/", unit.path.display()),
        (false, false) => unit.path.display().to_string(),
    }
}

fn unnamed(units: &Units<'_>, unit: &Unit<'_>, described: &Counts) -> Problem {
    let visible = units.visible(unit);
    let weight = |word: &String| units.idf.get(word).copied().unwrap_or(0.0);
    let total: f64 = described.keys().map(weight).sum();
    let covered: f64 = described
        .keys()
        .filter(|word| visible.contains(*word))
        .map(weight)
        .sum();
    let mut missing: Vec<&String> = described
        .keys()
        .filter(|word| !visible.contains(*word))
        .collect();
    missing.sort_by(|a, b| weight(b).total_cmp(&weight(a)).then_with(|| a.cmp(b)));
    let shown: Vec<String> = missing
        .iter()
        .take(KEY_TERMS)
        .map(|word| units.surface(word))
        .collect();
    let weighted = total > 0.0;
    let coverage = if weighted { covered / total } else { COVERED };
    Problem {
        value: 1.0 - (coverage / COVERED).min(1.0),
        kind: "rename",
        detail: format!("missing: {}", shown.join(" ")),
    }
}

fn hidden(units: &Units<'_>, unit: &Unit<'_>) -> Problem {
    let size = |source: &Source| real(source.words.values().sum());
    let (clusters, share) = if unit.directory {
        let files = units.corpus.files.iter();
        let direct = files.filter(|(file, _)| units.owner(file) == unit.path);
        let (mut inside, mut total, mut clusters) = (0.0, 0.0, 0);
        for (file, source) in direct {
            total += size(source);
            let entry = units.entries.contains_key(file);
            if entry {
                inside += size(source);
                clusters += source.clusters;
            }
        }
        let sized = total > 0.0;
        (clusters, if sized { inside / total } else { 0.0 })
    } else {
        let source = units.corpus.files.get(unit.path);
        (source.map_or(0, |source| source.clusters), 1.0)
    };
    let spread = (real(clusters.saturating_sub(1)) / real(CLUSTERS)).min(1.0);
    Problem {
        value: share * spread,
        kind: "split",
        detail: format!("clusters: {clusters}"),
    }
}

fn crowded(units: &Units<'_>, unit: &Unit<'_>) -> Problem {
    let count = if unit.directory {
        units.children.get(unit.path).map_or(0, Vec::len)
    } else {
        0
    };
    let over = count.saturating_sub(ENTRIES);
    Problem {
        value: (real(over) / real(ENTRIES)).min(1.0),
        kind: "split",
        detail: format!("entries: {count}"),
    }
}

fn misplaced(units: &Units<'_>, unit: &Unit<'_>, vector: &Vector) -> Problem {
    let parent = unit.path.parent().unwrap_or(Path::new(""));
    let mut rest = units.contents.get(parent).cloned().unwrap_or_default();
    for (word, count) in units.content(unit) {
        if let Some(slot) = rest.get_mut(word) {
            *slot = slot.saturating_sub(*count);
        }
    }
    let home = vector.cosine(&units.vector(&rest));
    let candidates = units.vectors.iter().filter(|(path, _)| {
        let related = unit.path.starts_with(path) || path.starts_with(unit.path);
        !related && path.as_path() != parent
    });
    let similar = candidates.map(|(path, other)| (path, vector.cosine(other)));
    let (target, value) = closest(similar).map_or((String::new(), 0.0), |(path, similarity)| {
        (path.display().to_string(), (similarity - home).max(0.0))
    });
    Problem {
        value,
        kind: "move",
        detail: format!("closer: {target}/"),
    }
}

fn duplicate(units: &Units<'_>, unit: &Unit<'_>, vector: &Vector) -> Problem {
    let parent = unit.path.parent();
    let siblings = parent
        .and_then(|parent| units.children.get(parent))
        .into_iter()
        .flatten();
    let others = siblings.filter(|(path, _)| path.as_path() != unit.path);
    let similar = others.map(|(path, directory)| {
        let sibling = Unit {
            path,
            directory: *directory,
        };
        (path, vector.cosine(&units.vector(units.content(&sibling))))
    });
    let (partner, value) = closest(similar).map_or((String::new(), 0.0), |(path, similarity)| {
        let file = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned());
        (
            file.unwrap_or_default(),
            ((similarity - SIMILAR) * 2.0).max(0.0),
        )
    });
    Problem {
        value,
        kind: "merge",
        detail: format!("close: {partner}"),
    }
}

fn closest<'p>(similar: impl Iterator<Item = (&'p PathBuf, f64)>) -> Option<(&'p PathBuf, f64)> {
    similar.fold(None, |best, next| match best {
        Some(current) if current.1 >= next.1 => Some(current),
        _ => Some(next),
    })
}

fn long(unit: &Unit<'_>) -> Problem {
    let count = name(unit).map_or(0, |name| split(&name).len());
    let over = count.saturating_sub(NAME_WORDS);
    Problem {
        value: (real(over) / real(NAME_WORDS)).min(1.0),
        kind: "rename",
        detail: format!("words: {count}"),
    }
}
