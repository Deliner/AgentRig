use super::corpus::{Corpus, Counts};
use super::words::stems;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const ENTRY_FILES: [&str; 3] = ["mod.rs", "lib.rs", "main.rs"];

pub struct Vector {
    weights: BTreeMap<String, f64>,
}

impl Vector {
    pub fn cosine(&self, other: &Self) -> f64 {
        let dot: f64 = self
            .weights
            .iter()
            .filter_map(|(word, weight)| other.weights.get(word).map(|value| weight * value))
            .sum();
        let denominator = self.norm() * other.norm();
        let measurable = denominator > 0.0;
        if measurable { dot / denominator } else { 0.0 }
    }

    fn norm(&self) -> f64 {
        let squares = self.weights.values().map(|weight| weight * weight);
        squares.sum::<f64>().sqrt()
    }
}

pub struct Unit<'a> {
    pub path: &'a Path,
    pub directory: bool,
}

pub struct Units<'a> {
    pub corpus: &'a Corpus,
    pub entries: BTreeMap<PathBuf, PathBuf>,
    pub children: BTreeMap<PathBuf, Vec<(PathBuf, bool)>>,
    pub contents: BTreeMap<PathBuf, Counts>,
    pub vectors: BTreeMap<PathBuf, Vector>,
    pub idf: BTreeMap<String, f64>,
}

impl<'a> Units<'a> {
    pub fn new(corpus: &'a Corpus) -> Self {
        let mut directories = BTreeSet::from([PathBuf::new()]);
        for file in corpus.files.keys() {
            directories.extend(file.ancestors().skip(1).map(Path::to_path_buf));
        }
        let entries: BTreeMap<PathBuf, PathBuf> = corpus
            .files
            .keys()
            .filter_map(|file| Some((file.clone(), entry_of(file, &directories)?)))
            .collect();
        let files = corpus
            .files
            .keys()
            .filter(|file| !entries.contains_key(*file));
        let children = layout(&directories, files);
        let mut units = Self {
            corpus,
            entries,
            children,
            contents: BTreeMap::new(),
            vectors: BTreeMap::new(),
            idf: idf(corpus),
        };
        units.contents = units.directory_contents();
        units.vectors = units
            .contents
            .iter()
            .map(|(path, counts)| (path.clone(), units.vector(counts)))
            .collect();
        units
    }

    pub fn all(&self) -> Vec<Unit<'_>> {
        let directories = self.children.keys().map(|path| Unit {
            path,
            directory: true,
        });
        let files = self.corpus.files.keys();
        let files = files.filter(|file| !self.entries.contains_key(*file));
        directories
            .chain(files.map(|path| Unit {
                path,
                directory: false,
            }))
            .collect()
    }

    fn directory_contents(&self) -> BTreeMap<PathBuf, Counts> {
        let mut contents: BTreeMap<PathBuf, Counts> = BTreeMap::new();
        for (file, source) in &self.corpus.files {
            for directory in self.owner(file).ancestors() {
                add(
                    contents.entry(directory.to_path_buf()).or_default(),
                    &source.words,
                );
            }
        }
        contents
    }

    pub fn owner<'p>(&'p self, file: &'p Path) -> &'p Path {
        self.entries
            .get(file)
            .map_or_else(|| file.parent().unwrap_or(Path::new("")), PathBuf::as_path)
    }

    pub fn described(&self, unit: &Unit<'_>) -> Counts {
        let mut words = self
            .corpus
            .descriptions
            .get(unit.path)
            .cloned()
            .unwrap_or_default();
        let documented = self.corpus.files.iter().filter(|(file, _)| {
            let owner = if unit.directory {
                self.entries.get(*file)
            } else {
                Some(*file)
            };
            owner.is_some_and(|owner| owner == unit.path)
        });
        for (_, source) in documented {
            add(&mut words, &source.about);
        }
        words
    }

    pub fn visible(&self, unit: &Unit<'_>) -> BTreeSet<String> {
        let entries = self.children.get(unit.path).into_iter().flatten();
        let names = entries
            .filter(|_| unit.directory)
            .map(|(path, directory)| Unit {
                path,
                directory: *directory,
            });
        let own = Unit {
            path: unit.path,
            directory: unit.directory,
        };
        std::iter::once(own)
            .chain(names)
            .filter_map(|unit| name(&unit))
            .flat_map(|name| stems(&name))
            .collect()
    }

    pub fn vector(&self, counts: &Counts) -> Vector {
        let weights: BTreeMap<String, f64> = counts
            .iter()
            .filter_map(|(word, &count)| {
                let weight = self.idf.get(word).copied().unwrap_or(0.0);
                (weight > 0.0).then(|| (word.clone(), (1.0 + real(count).ln()) * weight))
            })
            .collect();
        Vector { weights }
    }

    pub fn content(&self, unit: &Unit<'_>) -> &Counts {
        let source = (!unit.directory)
            .then(|| self.corpus.files.get(unit.path))
            .flatten();
        source.map_or_else(
            || self.contents.get(unit.path).unwrap_or(&EMPTY),
            |source| &source.words,
        )
    }

    pub fn surface(&self, word: &str) -> String {
        let forms = self.corpus.surfaces.get(word).into_iter().flatten();
        let best = forms.fold(
            None,
            |best: Option<(&String, usize)>, (form, &count)| match best {
                Some(current) if current.1 >= count => Some(current),
                _ => Some((form, count)),
            },
        );
        best.map_or_else(|| word.to_owned(), |(form, _)| form.clone())
    }
}

static EMPTY: Counts = BTreeMap::new();

fn layout<'p>(
    directories: &BTreeSet<PathBuf>,
    files: impl Iterator<Item = &'p PathBuf>,
) -> BTreeMap<PathBuf, Vec<(PathBuf, bool)>> {
    let mut children: BTreeMap<PathBuf, Vec<(PathBuf, bool)>> = directories
        .iter()
        .map(|path| (path.clone(), Vec::new()))
        .collect();
    let units = directories.iter().skip(1).map(|path| (path, true));
    for (path, directory) in units.chain(files.map(|path| (path, false))) {
        let parent = path.parent().unwrap_or(Path::new("")).to_path_buf();
        children
            .entry(parent)
            .or_default()
            .push((path.clone(), directory));
    }
    children
}

fn idf(corpus: &Corpus) -> BTreeMap<String, f64> {
    let documents: Vec<&Counts> = corpus
        .files
        .values()
        .map(|source| &source.words)
        .chain(corpus.descriptions.values())
        .collect();
    let mut frequency: BTreeMap<String, usize> = BTreeMap::new();
    for words in &documents {
        for word in words.keys() {
            *frequency.entry(word.clone()).or_default() += 1;
        }
    }
    let total = real(documents.len()) + 1.0;
    frequency
        .into_iter()
        .map(|(word, count)| (word, (total / (real(count) + 1.0)).ln()))
        .collect()
}

pub fn name(unit: &Unit<'_>) -> Option<String> {
    let name = if unit.directory {
        unit.path.file_name()
    } else {
        unit.path.file_stem()
    };
    name.map(|name| name.to_string_lossy().into_owned())
}

fn entry_of(file: &Path, directories: &BTreeSet<PathBuf>) -> Option<PathBuf> {
    let parent = file.parent().unwrap_or(Path::new(""));
    let named = file.file_name().and_then(|name| name.to_str());
    let standard_entry = named.is_some_and(|name| ENTRY_FILES.contains(&name));
    if standard_entry {
        return Some(parent.to_path_buf());
    }
    let sibling = parent.join(file.file_stem()?);
    directories.contains(&sibling).then_some(sibling)
}

fn add(total: &mut Counts, words: &Counts) {
    for (word, count) in words {
        *total.entry(word.clone()).or_default() += count;
    }
}

pub fn real(count: usize) -> f64 {
    u32::try_from(count).map_or(f64::from(u32::MAX), f64::from)
}
